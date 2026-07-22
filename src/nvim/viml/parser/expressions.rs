use crate::src::nvim::charset::{hex2nr, vim_str2nr};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::trans_special;
use crate::src::nvim::mbyte::{mb_copy_char, utf_char2bytes, utf_char2len, utfc_ptr2len_len};
use crate::src::nvim::memory::{xfree, xmalloc, xmallocz, xrealloc};
use crate::src::nvim::os::libc::{
    __assert_fail, abort, gettext, memchr, memcmp, memcpy, snprintf, strchr,
};
pub use crate::src::nvim::types::{
    expr_ast_node, expr_ast_node_data as C2Rust_Unnamed_21,
    expr_ast_node_data_ass as C2Rust_Unnamed_22, expr_ast_node_data_cmp as C2Rust_Unnamed_28,
    expr_ast_node_data_env as C2Rust_Unnamed_23, expr_ast_node_data_fig as C2Rust_Unnamed_31,
    expr_ast_node_data_fig_type_guesses as C2Rust_Unnamed_32,
    expr_ast_node_data_flt as C2Rust_Unnamed_26, expr_ast_node_data_num as C2Rust_Unnamed_27,
    expr_ast_node_data_opt as C2Rust_Unnamed_24, expr_ast_node_data_reg as C2Rust_Unnamed_33,
    expr_ast_node_data_str as C2Rust_Unnamed_25, expr_ast_node_data_ter as C2Rust_Unnamed_29,
    expr_ast_node_data_var as C2Rust_Unnamed_30, float_T, iconv_t, int64_t, size_t, uint64_t,
    uint8_t, uvarnumber_T, varnumber_T, vimconv_T, ExprAST, ExprASTError, ExprASTNode,
    ExprASTNodeType, ExprAssignmentType, ExprCaseCompareStrategy, ExprComparisonType, ExprOptScope,
    ExprParserFlags, ExprVarScope, ParserHighlight, ParserHighlightChunk, ParserInputReader,
    ParserInputReader_lines as C2Rust_Unnamed_5, ParserLine, ParserLineGetter, ParserPosition,
    ParserState, ParserStateItem, ParserStateItem_data as C2Rust_Unnamed_1,
    ParserStateItem_data_expr as C2Rust_Unnamed_2,
    ParserStateItem_data_expr_type_0 as C2Rust_Unnamed_3,
    ParserStateItem_type_0 as C2Rust_Unnamed_4, ParserState_stack as C2Rust_Unnamed_6,
};
use crate::src::nvim::viml::parser::parser::viml_parser_get_remaining_line;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const STR2NR_QUOTE: C2Rust_Unnamed = 16;
pub const STR2NR_NO_OCT: C2Rust_Unnamed = 13;
pub const STR2NR_ALL: C2Rust_Unnamed = 15;
pub const STR2NR_FORCE: C2Rust_Unnamed = 128;
pub const STR2NR_OOCT: C2Rust_Unnamed = 8;
pub const STR2NR_HEX: C2Rust_Unnamed = 4;
pub const STR2NR_OCT: C2Rust_Unnamed = 2;
pub const STR2NR_BIN: C2Rust_Unnamed = 1;
pub const STR2NR_DEC: C2Rust_Unnamed = 0;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const FSK_SIMPLIFY: C2Rust_Unnamed_0 = 8;
pub const FSK_IN_STRING: C2Rust_Unnamed_0 = 4;
pub const FSK_KEEP_X_KEY: C2Rust_Unnamed_0 = 2;
pub const FSK_KEYCODE: C2Rust_Unnamed_0 = 1;
pub const kExprUnknown: C2Rust_Unnamed_3 = 0;
pub const kPTopStateParsingExpression: C2Rust_Unnamed_4 = 1;
pub const kPTopStateParsingCommand: C2Rust_Unnamed_4 = 0;
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
#[repr(C)]
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExprASTStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut *mut ExprASTNode,
    pub init_array: [*mut *mut ExprASTNode; 16],
}
pub const kEPTLambdaArguments: ExprASTParseType = 1;
pub type ExprASTParseType = ::core::ffi::c_uint;
pub const kEPTSingleAssignment: ExprASTParseType = 3;
pub const kEPTAssignment: ExprASTParseType = 2;
pub const kEPTExpr: ExprASTParseType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExprASTParseTypeStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ExprASTParseType,
    pub init_array: [ExprASTParseType; 4],
}
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
#[repr(C)]
pub struct C2Rust_Unnamed_34 {
    pub lvl: ExprOpLvl,
    pub ass: ExprOpAssociativity,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StringShift {
    pub start: size_t,
    pub orig_len: size_t,
    pub act_len: size_t,
    pub escape_not_known: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_35 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut StringShift,
    pub init_array: [StringShift; 16],
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline(always)]
unsafe extern "C" fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dest, src, size);
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw const src as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    return dest;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = 9;
pub const NL: ::core::ffi::c_int = 10;
pub const Ctrl_A: ::core::ffi::c_int = 1;
pub const Ctrl_B: ::core::ffi::c_int = 2;
pub const Ctrl_C: ::core::ffi::c_int = 3;
pub const Ctrl_D: ::core::ffi::c_int = 4;
pub const Ctrl_E: ::core::ffi::c_int = 5;
pub const Ctrl_F: ::core::ffi::c_int = 6;
pub const Ctrl_G: ::core::ffi::c_int = 7;
pub const Ctrl_H: ::core::ffi::c_int = 8;
pub const Ctrl_K: ::core::ffi::c_int = 11;
pub const Ctrl_L: ::core::ffi::c_int = 12;
pub const Ctrl_M: ::core::ffi::c_int = 13;
pub const Ctrl_N: ::core::ffi::c_int = 14;
pub const Ctrl_O: ::core::ffi::c_int = 15;
pub const Ctrl_P: ::core::ffi::c_int = 16;
pub const Ctrl_Q: ::core::ffi::c_int = 17;
pub const Ctrl_R: ::core::ffi::c_int = 18;
pub const Ctrl_S: ::core::ffi::c_int = 19;
pub const Ctrl_T: ::core::ffi::c_int = 20;
pub const Ctrl_U: ::core::ffi::c_int = 21;
pub const Ctrl_V: ::core::ffi::c_int = 22;
pub const Ctrl_W: ::core::ffi::c_int = 23;
pub const Ctrl_X: ::core::ffi::c_int = 24;
pub const Ctrl_Y: ::core::ffi::c_int = 25;
pub const Ctrl_Z: ::core::ffi::c_int = 26;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isxdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int
        || c >= 'a' as ::core::ffi::c_int && c <= 'f' as ::core::ffi::c_int
        || c >= 'A' as ::core::ffi::c_int && c <= 'F' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isident(mut c: ::core::ffi::c_int) -> bool {
    return c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(c) as ::core::ffi::c_int != 0
        || c == '_' as ::core::ffi::c_int;
}
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn scale_number(
    num: float_T,
    base: uint8_t,
    exponent: uvarnumber_T,
    exponent_negative: bool,
) -> float_T {
    if num == 0 as ::core::ffi::c_int as float_T || exponent == 0 as uvarnumber_T {
        return num;
    }
    '_c2rust_label: {
        if base != 0 {
        } else {
            __assert_fail(
                b"base\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/viml/parser/expressions.rs\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                150 as ::core::ffi::c_uint,
                b"float_T scale_number(const float_T, const uint8_t, const uvarnumber_T, const _Bool)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut exp: uvarnumber_T = exponent;
    let mut p_base: float_T = base as float_T;
    let mut ret: float_T = num;
    while exp != 0 {
        if exp & 1 as uvarnumber_T != 0 {
            if exponent_negative {
                ret /= p_base;
            } else {
                ret *= p_base;
            }
        }
        exp >>= 1 as ::core::ffi::c_int;
        p_base *= p_base;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn viml_pexpr_next_token(
    pstate: *mut ParserState,
    flags: ::core::ffi::c_int,
) -> LexExprToken {
    let mut schar: uint8_t = 0;
    // The C partial initializer (`LexExprToken ret = { .type = ..., .start =
    // ... }`) zeroes the entire union; initializing only the `cmp` variant
    // leaves the tail of larger variants (e.g. `opt.scope`) as stack garbage,
    // which the parser later reads through for invalid option tokens.
    let mut ret: LexExprToken = ::core::mem::zeroed();
    ret.start = (*pstate).pos;
    ret.type_0 = kExprLexInvalid;
    let mut pline: ParserLine = ParserLine {
        data: ::core::ptr::null::<::core::ffi::c_char>(),
        size: 0,
        allocated: false,
    };
    if !viml_parser_get_remaining_line(pstate, &raw mut pline) {
        ret.type_0 = kExprLexEOC;
        return ret;
    }
    if pline.size <= 0 as size_t {
        ret.len = 0 as size_t;
        ret.type_0 = kExprLexEOC;
    } else {
        ret.len = 1 as size_t;
        schar = *pline.data.offset(0 as ::core::ffi::c_int as isize) as uint8_t;
        match schar as ::core::ffi::c_int {
            40 | 41 => {
                ret.type_0 = kExprLexParenthesis;
                ret.data.brc.closing = schar as ::core::ffi::c_int == ')' as ::core::ffi::c_int;
            }
            91 | 93 => {
                ret.type_0 = kExprLexBracket;
                ret.data.brc.closing = schar as ::core::ffi::c_int == ']' as ::core::ffi::c_int;
            }
            123 | 125 => {
                ret.type_0 = kExprLexFigureBrace;
                ret.data.brc.closing = schar as ::core::ffi::c_int == '}' as ::core::ffi::c_int;
            }
            63 => {
                ret.type_0 = kExprLexQuestion;
            }
            58 => {
                ret.type_0 = kExprLexColon;
            }
            44 => {
                ret.type_0 = kExprLexComma;
            }
            42 => {
                ret.type_0 = kExprLexMultiplication;
                ret.data.mul.type_0 = kExprLexMulMul;
            }
            47 => {
                ret.type_0 = kExprLexMultiplication;
                ret.data.mul.type_0 = kExprLexMulDiv;
            }
            37 => {
                ret.type_0 = kExprLexMultiplication;
                ret.data.mul.type_0 = kExprLexMulMod;
            }
            32 | TAB => {
                ret.type_0 = kExprLexSpacing;
                while ret.len < pline.size
                    && ascii_iswhite(*pline.data.offset(ret.len as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0
                {
                    ret.len = ret.len.wrapping_add(1);
                }
            }
            Ctrl_A | Ctrl_B | Ctrl_C | Ctrl_D | Ctrl_E | Ctrl_F | Ctrl_G | Ctrl_H | Ctrl_K
            | Ctrl_L | Ctrl_M | Ctrl_N | Ctrl_O | Ctrl_P | Ctrl_Q | Ctrl_R | Ctrl_S | Ctrl_T
            | Ctrl_U | Ctrl_V | Ctrl_W | Ctrl_X | Ctrl_Y | Ctrl_Z => {
                ret.type_0 = kExprLexInvalid;
                while ret.len < pline.size
                    && (*pline.data.offset(ret.len as isize) as ::core::ffi::c_int)
                        < ' ' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                }
                ret.data.err.type_0 = kExprLexSpacing;
                ret.data.err.msg = gettext(
                    b"E15: Invalid control character present in input: %.*s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
            48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                ret.data.num.is_float = false_0 != 0;
                ret.data.num.base = 10 as uint8_t;
                let mut frac_start: size_t = 0 as size_t;
                let mut exp_start: size_t = 0 as size_t;
                let mut frac_end: size_t = 0 as size_t;
                let mut exp_negative: bool = false_0 != 0;
                ret.type_0 = kExprLexNumber;
                while ret.len < pline.size
                    && ascii_isdigit(*pline.data.offset(ret.len as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0
                {
                    ret.len = ret.len.wrapping_add(1);
                }
                if flags & kELFlagAllowFloat as ::core::ffi::c_int != 0 {
                    let non_float_ret: LexExprToken = ret;
                    if pline.size > ret.len.wrapping_add(1 as size_t)
                        && *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                        && ascii_isdigit(
                            *pline
                                .data
                                .offset(ret.len.wrapping_add(1 as size_t) as isize)
                                as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        ret.len = ret.len.wrapping_add(1);
                        frac_start = ret.len;
                        frac_end = ret.len;
                        ret.data.num.is_float = true_0 != 0;
                        while ret.len < pline.size
                            && ascii_isdigit(
                                *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                            ) as ::core::ffi::c_int
                                != 0
                        {
                            if *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                                != '0' as ::core::ffi::c_int
                            {
                                frac_end = ret.len.wrapping_add(1 as size_t);
                            }
                            ret.len = ret.len.wrapping_add(1);
                        }
                        if pline.size > ret.len.wrapping_add(1 as size_t)
                            && (*pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                                == 'e' as ::core::ffi::c_int
                                || *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                                    == 'E' as ::core::ffi::c_int)
                            && (pline.size > ret.len.wrapping_add(2 as size_t)
                                && (*pline
                                    .data
                                    .offset(ret.len.wrapping_add(1 as size_t) as isize)
                                    as ::core::ffi::c_int
                                    == '+' as ::core::ffi::c_int
                                    || *pline
                                        .data
                                        .offset(ret.len.wrapping_add(1 as size_t) as isize)
                                        as ::core::ffi::c_int
                                        == '-' as ::core::ffi::c_int)
                                && ascii_isdigit(
                                    *pline
                                        .data
                                        .offset(ret.len.wrapping_add(2 as size_t) as isize)
                                        as ::core::ffi::c_int,
                                ) as ::core::ffi::c_int
                                    != 0
                                || ascii_isdigit(
                                    *pline
                                        .data
                                        .offset(ret.len.wrapping_add(1 as size_t) as isize)
                                        as ::core::ffi::c_int,
                                ) as ::core::ffi::c_int
                                    != 0)
                        {
                            ret.len = ret.len.wrapping_add(1);
                            if *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                                == '+' as ::core::ffi::c_int
                                || {
                                    exp_negative = *pline.data.offset(ret.len as isize)
                                        as ::core::ffi::c_int
                                        == '-' as ::core::ffi::c_int;
                                    exp_negative as ::core::ffi::c_int != 0
                                }
                            {
                                ret.len = ret.len.wrapping_add(1);
                            }
                            exp_start = ret.len;
                            ret.type_0 = kExprLexNumber;
                            while ret.len < pline.size
                                && ascii_isdigit(
                                    *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0
                            {
                                ret.len = ret.len.wrapping_add(1);
                            }
                        }
                    }
                    if pline.size > ret.len
                        && (*pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                            || (*pline.data.offset(ret.len as isize) as ::core::ffi::c_uint
                                >= 'A' as ::core::ffi::c_uint
                                && *pline.data.offset(ret.len as isize) as ::core::ffi::c_uint
                                    <= 'Z' as ::core::ffi::c_uint
                                || *pline.data.offset(ret.len as isize) as ::core::ffi::c_uint
                                    >= 'a' as ::core::ffi::c_uint
                                    && *pline.data.offset(ret.len as isize) as ::core::ffi::c_uint
                                        <= 'z' as ::core::ffi::c_uint))
                    {
                        ret = non_float_ret;
                    }
                }
                if ret.data.num.is_float {
                    let mut significand_part: float_T = 0 as ::core::ffi::c_int as float_T;
                    let mut exp_part: uvarnumber_T = 0 as uvarnumber_T;
                    let frac_size: size_t = frac_end.wrapping_sub(frac_start);
                    let mut i: size_t = 0 as size_t;
                    while i < frac_end {
                        if i != frac_start.wrapping_sub(1 as size_t) {
                            significand_part = significand_part
                                * 10 as ::core::ffi::c_int as float_T
                                + (*pline.data.offset(i as isize) as ::core::ffi::c_int
                                    - '0' as ::core::ffi::c_int)
                                    as float_T;
                        }
                        i = i.wrapping_add(1);
                    }
                    if exp_start != 0 {
                        vim_str2nr(
                            pline.data.offset(exp_start as isize),
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                            0 as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<varnumber_T>(),
                            &raw mut exp_part,
                            ret.len.wrapping_sub(exp_start) as ::core::ffi::c_int,
                            false_0 != 0,
                            ::core::ptr::null_mut::<bool>(),
                        );
                    }
                    if exp_negative {
                        exp_part = (exp_part as ::core::ffi::c_ulong)
                            .wrapping_add(frac_size as ::core::ffi::c_ulong)
                            as uvarnumber_T;
                    } else if exp_part < frac_size as uvarnumber_T {
                        exp_negative = true_0 != 0;
                        exp_part = frac_size.wrapping_sub(exp_part as size_t) as uvarnumber_T;
                    } else {
                        exp_part = (exp_part as ::core::ffi::c_ulong)
                            .wrapping_sub(frac_size as ::core::ffi::c_ulong)
                            as uvarnumber_T;
                    }
                    ret.data.num.val.floating =
                        scale_number(significand_part, 10 as uint8_t, exp_part, exp_negative);
                } else {
                    let mut len: ::core::ffi::c_int = 0;
                    let mut prep: ::core::ffi::c_int = 0;
                    vim_str2nr(
                        pline.data,
                        &raw mut prep,
                        &raw mut len,
                        STR2NR_ALL as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<varnumber_T>(),
                        &raw mut ret.data.num.val.integer,
                        pline.size as ::core::ffi::c_int,
                        false_0 != 0,
                        ::core::ptr::null_mut::<bool>(),
                    );
                    ret.len = len as size_t;
                    let bases: [uint8_t; 121] = [
                        10 as uint8_t,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        8 as uint8_t,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        2 as uint8_t,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        16 as uint8_t,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        2 as uint8_t,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        16 as uint8_t,
                    ];
                    ret.data.num.base = bases[prep as usize];
                }
            }
            36 => {
                ret.type_0 = kExprLexEnv;
                while ret.len < pline.size
                    && ascii_isident(*pline.data.offset(ret.len as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0
                {
                    ret.len = ret.len.wrapping_add(1);
                }
            }
            97 | 98 | 99 | 100 | 101 | 102 | 103 | 104 | 105 | 106 | 107 | 108 | 109 | 110
            | 111 | 112 | 113 | 114 | 115 | 116 | 117 | 118 | 119 | 120 | 121 | 122 | 65 | 66
            | 67 | 68 | 69 | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 80 | 81 | 82
            | 83 | 84 | 85 | 86 | 87 | 88 | 89 | 90 | 95 => {
                ret.data.var.scope = kExprVarScopeMissing;
                ret.data.var.autoload = false_0 != 0;
                ret.type_0 = kExprLexPlainIdentifier;
                while ret.len < pline.size
                    && ascii_isident(*pline.data.offset(ret.len as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0
                {
                    ret.len = ret.len.wrapping_add(1);
                }
                if flags & kELFlagIsNotCmp as ::core::ffi::c_int == 0
                    && (ret.len == 2 as size_t
                        && memcmp(
                            pline.data as *const ::core::ffi::c_void,
                            b"is\0".as_ptr() as *const ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        || ret.len == 5 as size_t
                            && memcmp(
                                pline.data as *const ::core::ffi::c_void,
                                b"isnot\0".as_ptr() as *const ::core::ffi::c_char
                                    as *const ::core::ffi::c_void,
                                5 as size_t,
                            ) == 0 as ::core::ffi::c_int)
                {
                    ret.type_0 = kExprLexComparison;
                    ret.data.cmp.type_0 = kExprCmpIdentical;
                    ret.data.cmp.inv = ret.len == 5 as size_t;
                    if ret.len < pline.size
                        && !strchr(
                            b"?#\0".as_ptr() as *const ::core::ffi::c_char,
                            *pline.data.offset(ret.len as isize) as ::core::ffi::c_int,
                        )
                        .is_null()
                    {
                        ret.data.cmp.ccs =
                            *pline.data.offset(ret.len as isize) as ExprCaseCompareStrategy;
                        ret.len = ret.len.wrapping_add(1);
                    } else {
                        ret.data.cmp.ccs = kCCStrategyUseOption;
                    }
                } else if ret.len == 1 as size_t
                    && pline.size > 1 as size_t
                    && {
                        let mut c2rust_lvalue: [::core::ffi::c_char; 9] = [
                            kExprVarScopeScript as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeVim as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeBuffer as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeWindow as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeTabpage as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeLocal as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeBuffer as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeArguments as ::core::ffi::c_int as ::core::ffi::c_char,
                        ];
                        !memchr(
                            &raw mut c2rust_lvalue as *mut ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            schar as ::core::ffi::c_int,
                            ::core::mem::size_of::<[::core::ffi::c_char; 9]>(),
                        )
                        .is_null()
                    }
                    && *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                    && flags & kELFlagForbidScope as ::core::ffi::c_int == 0
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.data.var.scope = schar as ExprVarScope;
                    ret.type_0 = kExprLexPlainIdentifier;
                    while ret.len < pline.size
                        && (ascii_isident(*pline.data.offset(ret.len as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                            || *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                                == AUTOLOAD_CHAR)
                    {
                        ret.len = ret.len.wrapping_add(1);
                    }
                    ret.data.var.autoload = !memchr(
                        pline.data.offset(2 as ::core::ffi::c_int as isize)
                            as *const ::core::ffi::c_void,
                        AUTOLOAD_CHAR,
                        ret.len.wrapping_sub(2 as size_t),
                    )
                    .is_null();
                } else if pline.size > ret.len
                    && *pline.data.offset(ret.len as isize) as ::core::ffi::c_int == AUTOLOAD_CHAR
                {
                    ret.data.var.autoload = true_0 != 0;
                    ret.type_0 = kExprLexPlainIdentifier;
                    while ret.len < pline.size
                        && (ascii_isident(*pline.data.offset(ret.len as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                            || *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                                == AUTOLOAD_CHAR)
                    {
                        ret.len = ret.len.wrapping_add(1);
                    }
                }
            }
            38 => {
                if pline.size > 1 as size_t
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '&' as ::core::ffi::c_int
                {
                    ret.type_0 = kExprLexAnd;
                    ret.len = ret.len.wrapping_add(1);
                } else if pline.size == 1 as size_t
                    || !(*pline.data.offset(1 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *pline.data.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *pline.data.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *pline.data.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint)
                {
                    ret.type_0 = kExprLexInvalid;
                    ret.data.err.type_0 = kExprLexOption;
                    ret.data.err.msg =
                        gettext(b"E112: Option name missing: %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char);
                } else {
                    ret.type_0 = kExprLexOption;
                    if pline.size > 2 as size_t
                        && *pline.data.offset(2 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == ':' as ::core::ffi::c_int
                        && {
                            let mut c2rust_lvalue_0: [::core::ffi::c_char; 2] = [
                                kExprOptScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_char,
                                kExprOptScopeLocal as ::core::ffi::c_int as ::core::ffi::c_char,
                            ];
                            !memchr(
                                &raw mut c2rust_lvalue_0 as *mut ::core::ffi::c_char
                                    as *const ::core::ffi::c_void,
                                *pline.data.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int,
                                ::core::mem::size_of::<[::core::ffi::c_char; 2]>(),
                            )
                            .is_null()
                        }
                    {
                        ret.len = ret.len.wrapping_add(2 as size_t);
                        ret.data.opt.scope =
                            *pline.data.offset(1 as ::core::ffi::c_int as isize) as ExprOptScope;
                        ret.data.opt.name = pline.data.offset(3 as ::core::ffi::c_int as isize);
                    } else {
                        ret.data.opt.scope = kExprOptScopeUnspecified;
                        ret.data.opt.name = pline.data.offset(1 as ::core::ffi::c_int as isize);
                    }
                    let mut p: *const ::core::ffi::c_char = ret.data.opt.name;
                    let e: *const ::core::ffi::c_char = pline.data.offset(pline.size as isize);
                    if e.offset_from(p) >= 4 as isize
                        && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 't' as ::core::ffi::c_int
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '_' as ::core::ffi::c_int
                    {
                        ret.data.opt.len = 4 as size_t;
                        ret.len = ret.len.wrapping_add(4 as size_t);
                    } else {
                        while p < e
                            && (*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                                && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                                || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                                    && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                        {
                            p = p.offset(1);
                        }
                        ret.data.opt.len = p.offset_from(ret.data.opt.name) as size_t;
                        if ret.data.opt.len == 0 as size_t {
                            ret.type_0 = kExprLexInvalid;
                            ret.data.err.type_0 = kExprLexOption;
                            ret.data.err.msg =
                                gettext(b"E112: Option name missing: %.*s\0".as_ptr()
                                    as *const ::core::ffi::c_char);
                        } else {
                            ret.len = ret.len.wrapping_add(ret.data.opt.len);
                        }
                    }
                }
            }
            64 => {
                ret.type_0 = kExprLexRegister;
                if pline.size > 1 as size_t {
                    ret.len = ret.len.wrapping_add(1);
                    ret.data.reg.name = *pline.data.offset(1 as ::core::ffi::c_int as isize)
                        as uint8_t as ::core::ffi::c_int;
                } else {
                    ret.data.reg.name = -1 as ::core::ffi::c_int;
                }
            }
            39 => {
                ret.type_0 = kExprLexSingleQuotedString;
                ret.data.str.closed = false_0 != 0;
                while ret.len < pline.size && !ret.data.str.closed {
                    if *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                        == '\'' as ::core::ffi::c_int
                    {
                        if ret.len.wrapping_add(1 as size_t) < pline.size
                            && *pline
                                .data
                                .offset(ret.len.wrapping_add(1 as size_t) as isize)
                                as ::core::ffi::c_int
                                == '\'' as ::core::ffi::c_int
                        {
                            ret.len = ret.len.wrapping_add(1);
                        } else {
                            ret.data.str.closed = true_0 != 0;
                        }
                    }
                    ret.len = ret.len.wrapping_add(1);
                }
            }
            34 => {
                ret.type_0 = kExprLexDoubleQuotedString;
                ret.data.str.closed = false_0 != 0;
                while ret.len < pline.size && !ret.data.str.closed {
                    if *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                    {
                        if ret.len.wrapping_add(1 as size_t) < pline.size {
                            ret.len = ret.len.wrapping_add(1);
                        }
                    } else if *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                    {
                        ret.data.str.closed = true_0 != 0;
                    }
                    ret.len = ret.len.wrapping_add(1);
                }
            }
            33 | 61 => {
                if pline.size == 1 as size_t {
                    ret.type_0 = (if schar as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                        kExprLexNot as ::core::ffi::c_int
                    } else {
                        kExprLexAssignment as ::core::ffi::c_int
                    }) as LexExprTokenType;
                    ret.data.ass.type_0 = kExprAsgnPlain;
                } else {
                    ret.type_0 = kExprLexComparison;
                    ret.data.cmp.inv = schar as ::core::ffi::c_int == '!' as ::core::ffi::c_int;
                    if *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int
                    {
                        ret.data.cmp.type_0 = kExprCmpEqual;
                        ret.len = ret.len.wrapping_add(1);
                    } else if *pline.data.offset(1 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == '~' as ::core::ffi::c_int
                    {
                        ret.data.cmp.type_0 = kExprCmpMatches;
                        ret.len = ret.len.wrapping_add(1);
                    } else if schar as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                        ret.type_0 = kExprLexNot;
                    } else {
                        ret.type_0 = kExprLexAssignment;
                        ret.data.ass.type_0 = kExprAsgnPlain;
                    }
                    if ret.len < pline.size
                        && !strchr(
                            b"?#\0".as_ptr() as *const ::core::ffi::c_char,
                            *pline.data.offset(ret.len as isize) as ::core::ffi::c_int,
                        )
                        .is_null()
                    {
                        ret.data.cmp.ccs =
                            *pline.data.offset(ret.len as isize) as ExprCaseCompareStrategy;
                        ret.len = ret.len.wrapping_add(1);
                    } else {
                        ret.data.cmp.ccs = kCCStrategyUseOption;
                    }
                }
            }
            62 | 60 => {
                ret.type_0 = kExprLexComparison;
                let haseqsign: bool = pline.size > 1 as size_t
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int;
                if haseqsign {
                    ret.len = ret.len.wrapping_add(1);
                }
                if ret.len < pline.size
                    && !strchr(
                        b"?#\0".as_ptr() as *const ::core::ffi::c_char,
                        *pline.data.offset(ret.len as isize) as ::core::ffi::c_int,
                    )
                    .is_null()
                {
                    ret.data.cmp.ccs =
                        *pline.data.offset(ret.len as isize) as ExprCaseCompareStrategy;
                    ret.len = ret.len.wrapping_add(1);
                } else {
                    ret.data.cmp.ccs = kCCStrategyUseOption;
                }
                ret.data.cmp.inv = schar as ::core::ffi::c_int == '<' as ::core::ffi::c_int;
                ret.data.cmp.type_0 = (if ret.data.cmp.inv as ::core::ffi::c_int
                    ^ haseqsign as ::core::ffi::c_int
                    != 0
                {
                    kExprCmpGreaterOrEqual as ::core::ffi::c_int
                } else {
                    kExprCmpGreater as ::core::ffi::c_int
                }) as ExprComparisonType;
            }
            45 => {
                if pline.size > 1 as size_t
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '>' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.type_0 = kExprLexArrow;
                } else if pline.size > 1 as size_t
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.type_0 = kExprLexAssignment;
                    ret.data.ass.type_0 = kExprAsgnSubtract;
                } else {
                    ret.type_0 = kExprLexMinus;
                }
            }
            43 => {
                if pline.size > 1 as size_t
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.type_0 = kExprLexAssignment;
                    ret.data.ass.type_0 = kExprAsgnAdd;
                } else {
                    ret.type_0 = kExprLexPlus;
                }
            }
            46 => {
                if pline.size > 1 as size_t
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.type_0 = kExprLexAssignment;
                    ret.data.ass.type_0 = kExprAsgnConcat;
                } else {
                    ret.type_0 = kExprLexDot;
                }
            }
            NUL | NL => {
                if flags & kELFlagForbidEOC as ::core::ffi::c_int != 0 {
                    ret.type_0 = kExprLexInvalid;
                    ret.data.err.msg = gettext(b"E15: Unexpected EOC character: %.*s\0".as_ptr()
                        as *const ::core::ffi::c_char);
                    ret.data.err.type_0 = kExprLexSpacing;
                } else {
                    ret.type_0 = kExprLexEOC;
                }
            }
            124 => {
                if pline.size >= 2 as size_t
                    && *pline.data.offset(ret.len as isize) as ::core::ffi::c_int
                        == '|' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.type_0 = kExprLexOr;
                } else if flags & kELFlagForbidEOC as ::core::ffi::c_int != 0 {
                    ret.type_0 = kExprLexInvalid;
                    ret.data.err.msg = gettext(b"E15: Unexpected EOC character: %.*s\0".as_ptr()
                        as *const ::core::ffi::c_char);
                    ret.data.err.type_0 = kExprLexOr;
                } else {
                    ret.type_0 = kExprLexEOC;
                }
            }
            _ => {
                ret.len = utfc_ptr2len_len(pline.data, pline.size as ::core::ffi::c_int) as size_t;
                ret.type_0 = kExprLexInvalid;
                ret.data.err.type_0 = kExprLexPlainIdentifier;
                ret.data.err.msg =
                    gettext(b"E15: Unidentified character: %.*s\0".as_ptr()
                        as *const ::core::ffi::c_char);
            }
        }
    }
    if flags & kELFlagPeek as ::core::ffi::c_int == 0 {
        viml_parser_advance(pstate, ret.len);
    }
    return ret;
}
static eltkn_type_tab: GlobalCell<[*const ::core::ffi::c_char; 27]> = GlobalCell::new([
    b"Invalid\0".as_ptr() as *const ::core::ffi::c_char,
    b"Missing\0".as_ptr() as *const ::core::ffi::c_char,
    b"Spacing\0".as_ptr() as *const ::core::ffi::c_char,
    b"EOC\0".as_ptr() as *const ::core::ffi::c_char,
    b"Question\0".as_ptr() as *const ::core::ffi::c_char,
    b"Colon\0".as_ptr() as *const ::core::ffi::c_char,
    b"Or\0".as_ptr() as *const ::core::ffi::c_char,
    b"And\0".as_ptr() as *const ::core::ffi::c_char,
    b"Comparison\0".as_ptr() as *const ::core::ffi::c_char,
    b"Plus\0".as_ptr() as *const ::core::ffi::c_char,
    b"Minus\0".as_ptr() as *const ::core::ffi::c_char,
    b"Dot\0".as_ptr() as *const ::core::ffi::c_char,
    b"Multiplication\0".as_ptr() as *const ::core::ffi::c_char,
    b"Not\0".as_ptr() as *const ::core::ffi::c_char,
    b"Number\0".as_ptr() as *const ::core::ffi::c_char,
    b"SingleQuotedString\0".as_ptr() as *const ::core::ffi::c_char,
    b"DoubleQuotedString\0".as_ptr() as *const ::core::ffi::c_char,
    b"Option\0".as_ptr() as *const ::core::ffi::c_char,
    b"Register\0".as_ptr() as *const ::core::ffi::c_char,
    b"Env\0".as_ptr() as *const ::core::ffi::c_char,
    b"PlainIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
    b"Bracket\0".as_ptr() as *const ::core::ffi::c_char,
    b"FigureBrace\0".as_ptr() as *const ::core::ffi::c_char,
    b"Parenthesis\0".as_ptr() as *const ::core::ffi::c_char,
    b"Comma\0".as_ptr() as *const ::core::ffi::c_char,
    b"Arrow\0".as_ptr() as *const ::core::ffi::c_char,
    b"Assignment\0".as_ptr() as *const ::core::ffi::c_char,
]);
#[no_mangle]
pub static eltkn_cmp_type_tab: GlobalCell<[*const ::core::ffi::c_char; 5]> = GlobalCell::new([
    b"Equal\0".as_ptr() as *const ::core::ffi::c_char,
    b"Matches\0".as_ptr() as *const ::core::ffi::c_char,
    b"Greater\0".as_ptr() as *const ::core::ffi::c_char,
    b"GreaterOrEqual\0".as_ptr() as *const ::core::ffi::c_char,
    b"Identical\0".as_ptr() as *const ::core::ffi::c_char,
]);
#[no_mangle]
pub static expr_asgn_type_tab: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"Plain\0".as_ptr() as *const ::core::ffi::c_char,
    b"Add\0".as_ptr() as *const ::core::ffi::c_char,
    b"Subtract\0".as_ptr() as *const ::core::ffi::c_char,
    b"Concat\0".as_ptr() as *const ::core::ffi::c_char,
]);
#[no_mangle]
pub static ccs_tab: GlobalCell<[*const ::core::ffi::c_char; 64]> = GlobalCell::new([
    b"UseOption\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"MatchCase\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"IgnoreCase\0".as_ptr() as *const ::core::ffi::c_char,
]);
static eltkn_mul_type_tab: GlobalCell<[*const ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"Mul\0".as_ptr() as *const ::core::ffi::c_char,
    b"Div\0".as_ptr() as *const ::core::ffi::c_char,
    b"Mod\0".as_ptr() as *const ::core::ffi::c_char,
]);
static eltkn_opt_scope_tab: GlobalCell<[*const ::core::ffi::c_char; 109]> = GlobalCell::new([
    b"Unspecified\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"Global\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"Local\0".as_ptr() as *const ::core::ffi::c_char,
]);
#[no_mangle]
pub static east_node_type_tab: GlobalCell<[*const ::core::ffi::c_char; 39]> = GlobalCell::new([
    b"Missing\0".as_ptr() as *const ::core::ffi::c_char,
    b"OpMissing\0".as_ptr() as *const ::core::ffi::c_char,
    b"Ternary\0".as_ptr() as *const ::core::ffi::c_char,
    b"TernaryValue\0".as_ptr() as *const ::core::ffi::c_char,
    b"Register\0".as_ptr() as *const ::core::ffi::c_char,
    b"Subscript\0".as_ptr() as *const ::core::ffi::c_char,
    b"ListLiteral\0".as_ptr() as *const ::core::ffi::c_char,
    b"UnaryPlus\0".as_ptr() as *const ::core::ffi::c_char,
    b"BinaryPlus\0".as_ptr() as *const ::core::ffi::c_char,
    b"Nested\0".as_ptr() as *const ::core::ffi::c_char,
    b"Call\0".as_ptr() as *const ::core::ffi::c_char,
    b"PlainIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
    b"PlainKey\0".as_ptr() as *const ::core::ffi::c_char,
    b"ComplexIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
    b"UnknownFigure\0".as_ptr() as *const ::core::ffi::c_char,
    b"Lambda\0".as_ptr() as *const ::core::ffi::c_char,
    b"DictLiteral\0".as_ptr() as *const ::core::ffi::c_char,
    b"CurlyBracesIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
    b"Comma\0".as_ptr() as *const ::core::ffi::c_char,
    b"Colon\0".as_ptr() as *const ::core::ffi::c_char,
    b"Arrow\0".as_ptr() as *const ::core::ffi::c_char,
    b"Comparison\0".as_ptr() as *const ::core::ffi::c_char,
    b"Concat\0".as_ptr() as *const ::core::ffi::c_char,
    b"ConcatOrSubscript\0".as_ptr() as *const ::core::ffi::c_char,
    b"Integer\0".as_ptr() as *const ::core::ffi::c_char,
    b"Float\0".as_ptr() as *const ::core::ffi::c_char,
    b"SingleQuotedString\0".as_ptr() as *const ::core::ffi::c_char,
    b"DoubleQuotedString\0".as_ptr() as *const ::core::ffi::c_char,
    b"Or\0".as_ptr() as *const ::core::ffi::c_char,
    b"And\0".as_ptr() as *const ::core::ffi::c_char,
    b"UnaryMinus\0".as_ptr() as *const ::core::ffi::c_char,
    b"BinaryMinus\0".as_ptr() as *const ::core::ffi::c_char,
    b"Not\0".as_ptr() as *const ::core::ffi::c_char,
    b"Multiplication\0".as_ptr() as *const ::core::ffi::c_char,
    b"Division\0".as_ptr() as *const ::core::ffi::c_char,
    b"Mod\0".as_ptr() as *const ::core::ffi::c_char,
    b"Option\0".as_ptr() as *const ::core::ffi::c_char,
    b"Environment\0".as_ptr() as *const ::core::ffi::c_char,
    b"Assignment\0".as_ptr() as *const ::core::ffi::c_char,
]);
unsafe extern "C" fn intchar2str(ch: ::core::ffi::c_int) -> *const ::core::ffi::c_char {
    static buf: GlobalCell<[::core::ffi::c_char; 13]> = GlobalCell::new([0; 13]);
    if ' ' as ::core::ffi::c_int <= ch && ch < 0x7f as ::core::ffi::c_int {
        if ascii_isdigit(ch) {
            (*buf.ptr())[0 as ::core::ffi::c_int as usize] = '\'' as ::core::ffi::c_char;
            (*buf.ptr())[1 as ::core::ffi::c_int as usize] = ch as ::core::ffi::c_char;
            (*buf.ptr())[2 as ::core::ffi::c_int as usize] = '\'' as ::core::ffi::c_char;
            (*buf.ptr())[3 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        } else {
            (*buf.ptr())[0 as ::core::ffi::c_int as usize] = ch as ::core::ffi::c_char;
            (*buf.ptr())[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        }
    } else {
        snprintf(
            buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 13]>(),
            b"%i\0".as_ptr() as *const ::core::ffi::c_char,
            ch,
        );
    }
    return buf.ptr() as *mut ::core::ffi::c_char;
}
pub static node_maxchildren: GlobalCell<[uint8_t; 39]> = GlobalCell::new([
    0 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
]);
#[no_mangle]
pub unsafe extern "C" fn viml_pexpr_free_ast(mut ast: ExprAST) {
    let mut ast_stack: ExprASTStack = ExprASTStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<*mut *mut ExprASTNode>(),
        init_array: [::core::ptr::null_mut::<*mut ExprASTNode>(); 16],
    };
    ast_stack.capacity = ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
        .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
        .wrapping_div(
            (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    ast_stack.size = 0 as size_t;
    ast_stack.items = &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode;
    if ast_stack.size == ast_stack.capacity {
        ast_stack.capacity = if ast_stack.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                .wrapping_div(
                    (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            ast_stack.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                .wrapping_div(
                    (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        ast_stack.items = (if ast_stack.capacity
            == ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                .wrapping_div(
                    (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if ast_stack.items == &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode {
                ast_stack.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                        as *mut ::core::ffi::c_void,
                    ast_stack.items as *mut ::core::ffi::c_void,
                    ast_stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                )
            }
        } else {
            if ast_stack.items == &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode {
                memcpy(
                    xmalloc(
                        ast_stack
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    ),
                    ast_stack.items as *const ::core::ffi::c_void,
                    ast_stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                )
            } else {
                xrealloc(
                    ast_stack.items as *mut ::core::ffi::c_void,
                    ast_stack
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                )
            }
        }) as *mut *mut *mut ExprASTNode;
    } else {
    };
    let c2rust_fresh1 = ast_stack.size;
    ast_stack.size = ast_stack.size.wrapping_add(1);
    let c2rust_lvalue_ptr = &raw mut *ast_stack.items.offset(c2rust_fresh1 as isize);
    *c2rust_lvalue_ptr = &raw mut ast.root;
    while ast_stack.size != 0 {
        let cur_node: *mut *mut ExprASTNode = *ast_stack.items.offset(
            ast_stack
                .size
                .wrapping_sub(0 as size_t)
                .wrapping_sub(1 as size_t) as isize,
        );
        let mut i: size_t = 0 as size_t;
        while i < ast_stack.size.wrapping_sub(1 as size_t) {
            '_c2rust_label: {
                if **ast_stack.items.offset(i as isize) != *cur_node {
                } else {
                    __assert_fail(
                        b"*kv_A(ast_stack, i) != *cur_node\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/viml/parser/expressions.rs\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        1035 as ::core::ffi::c_uint,
                        b"void viml_pexpr_free_ast(ExprAST)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            i = i.wrapping_add(1);
        }
        if (*cur_node).is_null() {
            '_c2rust_label_0: {
                if ast_stack.size == 1 as size_t {
                } else {
                    __assert_fail(
                        b"kv_size(ast_stack) == 1\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/viml/parser/expressions.rs\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        1039 as ::core::ffi::c_uint,
                        b"void viml_pexpr_free_ast(ExprAST)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            ast_stack.size = ast_stack.size.wrapping_sub(1 as size_t);
        } else if !(**cur_node).children.is_null() {
            let maxchildren: uint8_t = (*node_maxchildren.ptr())[(**cur_node).type_0 as usize];
            '_c2rust_label_1: {
                if maxchildren as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"maxchildren > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/viml/parser/expressions.rs\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        1044 as ::core::ffi::c_uint,
                        b"void viml_pexpr_free_ast(ExprAST)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            '_c2rust_label_2: {
                if maxchildren as ::core::ffi::c_int <= 2 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"maxchildren <= 2\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/viml/parser/expressions.rs\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        1045 as ::core::ffi::c_uint,
                        b"void viml_pexpr_free_ast(ExprAST)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            '_c2rust_label_3: {
                if (if maxchildren as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                    (*(**cur_node).children).next.is_null() as ::core::ffi::c_int
                } else {
                    ((*(**cur_node).children).next.is_null()
                        || (*(*(**cur_node).children).next).next.is_null())
                        as ::core::ffi::c_int
                }) != 0
                {
                } else {
                    __assert_fail(
                        b"maxchildren == 1 ? (*cur_node)->children->next == NULL : ((*cur_node)->children->next == NULL || (*cur_node)->children->next->next == NULL)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/viml/parser/expressions.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1049 as ::core::ffi::c_uint,
                        b"void viml_pexpr_free_ast(ExprAST)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if ast_stack.size == ast_stack.capacity {
                ast_stack.capacity = if ast_stack.capacity << 1 as ::core::ffi::c_int
                    > ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        .wrapping_div(
                            (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    ast_stack.capacity << 1 as ::core::ffi::c_int
                } else {
                    ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        .wrapping_div(
                            (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                                == 0) as ::core::ffi::c_int as size_t,
                        )
                };
                ast_stack.items = (if ast_stack.capacity
                    == ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        .wrapping_div(
                            (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    if ast_stack.items
                        == &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                    {
                        ast_stack.items as *mut ::core::ffi::c_void
                    } else {
                        _memcpy_free(
                            &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                                as *mut ::core::ffi::c_void,
                            ast_stack.items as *mut ::core::ffi::c_void,
                            ast_stack
                                .size
                                .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                        )
                    }
                } else {
                    if ast_stack.items
                        == &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                    {
                        memcpy(
                            xmalloc(
                                ast_stack
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                            ),
                            ast_stack.items as *const ::core::ffi::c_void,
                            ast_stack
                                .size
                                .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                        )
                    } else {
                        xrealloc(
                            ast_stack.items as *mut ::core::ffi::c_void,
                            ast_stack
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                        )
                    }
                }) as *mut *mut *mut ExprASTNode;
            } else {
            };
            let c2rust_fresh2 = ast_stack.size;
            ast_stack.size = ast_stack.size.wrapping_add(1);
            let c2rust_lvalue_ptr_0 = &raw mut *ast_stack.items.offset(c2rust_fresh2 as isize);
            *c2rust_lvalue_ptr_0 = &raw mut (**cur_node).children;
        } else if !(**cur_node).next.is_null() {
            if ast_stack.size == ast_stack.capacity {
                ast_stack.capacity = if ast_stack.capacity << 1 as ::core::ffi::c_int
                    > ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        .wrapping_div(
                            (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    ast_stack.capacity << 1 as ::core::ffi::c_int
                } else {
                    ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        .wrapping_div(
                            (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                                == 0) as ::core::ffi::c_int as size_t,
                        )
                };
                ast_stack.items = (if ast_stack.capacity
                    == ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        .wrapping_div(
                            (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    if ast_stack.items
                        == &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                    {
                        ast_stack.items as *mut ::core::ffi::c_void
                    } else {
                        _memcpy_free(
                            &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                                as *mut ::core::ffi::c_void,
                            ast_stack.items as *mut ::core::ffi::c_void,
                            ast_stack
                                .size
                                .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                        )
                    }
                } else {
                    if ast_stack.items
                        == &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                    {
                        memcpy(
                            xmalloc(
                                ast_stack
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                            ),
                            ast_stack.items as *const ::core::ffi::c_void,
                            ast_stack
                                .size
                                .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                        )
                    } else {
                        xrealloc(
                            ast_stack.items as *mut ::core::ffi::c_void,
                            ast_stack
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                        )
                    }
                }) as *mut *mut *mut ExprASTNode;
            } else {
            };
            let c2rust_fresh3 = ast_stack.size;
            ast_stack.size = ast_stack.size.wrapping_add(1);
            let c2rust_lvalue_ptr_1 = &raw mut *ast_stack.items.offset(c2rust_fresh3 as isize);
            *c2rust_lvalue_ptr_1 = &raw mut (**cur_node).next;
        } else if !(*cur_node).is_null() {
            ast_stack.size = ast_stack.size.wrapping_sub(1 as size_t);
            match (**cur_node).type_0 as ::core::ffi::c_uint {
                27 | 26 => {
                    xfree((**cur_node).data.str.value as *mut ::core::ffi::c_void);
                }
                0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17
                | 38 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 28 | 29 | 30 | 31 | 32 | 33 | 34
                | 35 | 36 | 37 | _ => {}
            }
            xfree(*cur_node as *mut ::core::ffi::c_void);
            *cur_node = ::core::ptr::null_mut::<ExprASTNode>();
        }
    }
    if ast_stack.items != &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut ast_stack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
}
#[inline]
unsafe extern "C" fn viml_pexpr_new_node(type_0: ExprASTNodeType) -> *mut ExprASTNode {
    let mut ret: *mut ExprASTNode =
        xmalloc(::core::mem::size_of::<ExprASTNode>()) as *mut ExprASTNode;
    (*ret).type_0 = type_0;
    (*ret).children = ::core::ptr::null_mut::<ExprASTNode>();
    (*ret).next = ::core::ptr::null_mut::<ExprASTNode>();
    return ret;
}
static node_type_to_node_props: GlobalCell<[C2Rust_Unnamed_34; 39]> = GlobalCell::new([
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlInvalid,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlTernary,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlTernaryValue,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlComplexIdentifier,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlComma,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlColon,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlArrow,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlComparison,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlSubscript,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlOr,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAnd,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAssignment,
        ass: kEOpAssLeft,
    },
]);
#[inline(always)]
unsafe extern "C" fn node_lvl(node: ExprASTNode) -> ExprOpLvl {
    return (*node_type_to_node_props.ptr())[node.type_0 as usize].lvl;
}
#[inline(always)]
unsafe extern "C" fn node_ass(node: ExprASTNode) -> ExprOpAssociativity {
    return (*node_type_to_node_props.ptr())[node.type_0 as usize].ass;
}
unsafe extern "C" fn viml_pexpr_handle_bop(
    pstate: *const ParserState,
    ast_stack: *mut ExprASTStack,
    bop_node: *mut ExprASTNode,
    want_node_p: *mut ExprASTWantedNode,
    ast_err: *mut ExprASTError,
) -> bool {
    let mut ret: bool = true_0 != 0;
    let mut top_node_p: *mut *mut ExprASTNode = ::core::ptr::null_mut::<*mut ExprASTNode>();
    let mut top_node: *mut ExprASTNode = ::core::ptr::null_mut::<ExprASTNode>();
    let mut top_node_lvl: ExprOpLvl = kEOpLvlInvalid;
    let mut top_node_ass: ExprOpAssociativity = 0 as ExprOpAssociativity;
    '_c2rust_label: {
        if (*ast_stack).size != 0 {
        } else {
            __assert_fail(
                b"kv_size(*ast_stack)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/viml/parser/expressions.rs\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                1260 as ::core::ffi::c_uint,
                b"_Bool viml_pexpr_handle_bop(const ParserState *const, ExprASTStack *const, ExprASTNode *const, ExprASTWantedNode *const, ExprASTError *const)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let bop_node_lvl: ExprOpLvl = (if (*bop_node).type_0 as ::core::ffi::c_uint
        == kExprNodeCall as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*bop_node).type_0 as ::core::ffi::c_uint
            == kExprNodeSubscript as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        kEOpLvlSubscript as ::core::ffi::c_int as ::core::ffi::c_uint
    } else {
        node_lvl(*bop_node) as ::core::ffi::c_uint
    }) as ExprOpLvl;
    loop {
        let mut new_top_node_p: *mut *mut ExprASTNode = *(*ast_stack).items.offset(
            (*ast_stack)
                .size
                .wrapping_sub(0 as size_t)
                .wrapping_sub(1 as size_t) as isize,
        );
        let mut new_top_node: *mut ExprASTNode = *new_top_node_p;
        '_c2rust_label_0: {
            if !new_top_node.is_null() {
            } else {
                __assert_fail(
                    b"new_top_node != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/viml/parser/expressions.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    1268 as ::core::ffi::c_uint,
                    b"_Bool viml_pexpr_handle_bop(const ParserState *const, ExprASTStack *const, ExprASTNode *const, ExprASTWantedNode *const, ExprASTError *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let new_top_node_lvl: ExprOpLvl = node_lvl(*new_top_node);
        let new_top_node_ass: ExprOpAssociativity = node_ass(*new_top_node);
        if !top_node_p.is_null()
            && (bop_node_lvl as ::core::ffi::c_uint > new_top_node_lvl as ::core::ffi::c_uint
                || bop_node_lvl as ::core::ffi::c_uint == new_top_node_lvl as ::core::ffi::c_uint
                    && new_top_node_ass as ::core::ffi::c_uint
                        == kEOpAssNo as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            break;
        }
        (*ast_stack).size = (*ast_stack).size.wrapping_sub(1 as size_t);
        top_node_p = new_top_node_p;
        top_node = new_top_node;
        top_node_lvl = new_top_node_lvl;
        top_node_ass = new_top_node_ass;
        if bop_node_lvl as ::core::ffi::c_uint == top_node_lvl as ::core::ffi::c_uint
            && top_node_ass as ::core::ffi::c_uint
                == kEOpAssRight as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        if (*ast_stack).size == 0 {
            break;
        }
    }
    if top_node_ass as ::core::ffi::c_uint
        == kEOpAssLeft as ::core::ffi::c_int as ::core::ffi::c_uint
        || top_node_lvl as ::core::ffi::c_uint != bop_node_lvl as ::core::ffi::c_uint
    {
        *top_node_p = bop_node;
        (*bop_node).children = top_node;
        '_c2rust_label_1: {
            if (*(*bop_node).children).next.is_null() {
            } else {
                __assert_fail(
                    b"bop_node->children->next == NULL\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/viml/parser/expressions.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    1296 as ::core::ffi::c_uint,
                    b"_Bool viml_pexpr_handle_bop(const ParserState *const, ExprASTStack *const, ExprASTNode *const, ExprASTWantedNode *const, ExprASTError *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if (*ast_stack).size == (*ast_stack).capacity {
            (*ast_stack).capacity = if (*ast_stack).capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*ast_stack).capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*ast_stack).items = (if (*ast_stack).capacity
                == ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*ast_stack).items
                    == &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                {
                    (*ast_stack).items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                            as *mut ::core::ffi::c_void,
                        (*ast_stack).items as *mut ::core::ffi::c_void,
                        (*ast_stack)
                            .size
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                }
            } else {
                if (*ast_stack).items
                    == &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                {
                    memcpy(
                        xmalloc(
                            (*ast_stack)
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                        ),
                        (*ast_stack).items as *const ::core::ffi::c_void,
                        (*ast_stack)
                            .size
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                } else {
                    xrealloc(
                        (*ast_stack).items as *mut ::core::ffi::c_void,
                        (*ast_stack)
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                }
            }) as *mut *mut *mut ExprASTNode;
        } else {
        };
        let c2rust_fresh29 = (*ast_stack).size;
        (*ast_stack).size = (*ast_stack).size.wrapping_add(1);
        let c2rust_lvalue_ptr = &raw mut *(*ast_stack).items.offset(c2rust_fresh29 as isize);
        *c2rust_lvalue_ptr = top_node_p;
        if (*ast_stack).size == (*ast_stack).capacity {
            (*ast_stack).capacity = if (*ast_stack).capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*ast_stack).capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*ast_stack).items = (if (*ast_stack).capacity
                == ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*ast_stack).items
                    == &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                {
                    (*ast_stack).items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                            as *mut ::core::ffi::c_void,
                        (*ast_stack).items as *mut ::core::ffi::c_void,
                        (*ast_stack)
                            .size
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                }
            } else {
                if (*ast_stack).items
                    == &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                {
                    memcpy(
                        xmalloc(
                            (*ast_stack)
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                        ),
                        (*ast_stack).items as *const ::core::ffi::c_void,
                        (*ast_stack)
                            .size
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                } else {
                    xrealloc(
                        (*ast_stack).items as *mut ::core::ffi::c_void,
                        (*ast_stack)
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                }
            }) as *mut *mut *mut ExprASTNode;
        } else {
        };
        let c2rust_fresh30 = (*ast_stack).size;
        (*ast_stack).size = (*ast_stack).size.wrapping_add(1);
        let c2rust_lvalue_ptr_0 = &raw mut *(*ast_stack).items.offset(c2rust_fresh30 as isize);
        *c2rust_lvalue_ptr_0 = &raw mut (*(*bop_node).children).next;
    } else {
        '_c2rust_label_2: {
            if top_node_lvl as ::core::ffi::c_uint == bop_node_lvl as ::core::ffi::c_uint
                && top_node_ass as ::core::ffi::c_uint
                    == kEOpAssRight as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"top_node_lvl == bop_node_lvl && top_node_ass == kEOpAssRight\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/viml/parser/expressions.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    1300 as ::core::ffi::c_uint,
                    b"_Bool viml_pexpr_handle_bop(const ParserState *const, ExprASTStack *const, ExprASTNode *const, ExprASTWantedNode *const, ExprASTError *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        '_c2rust_label_3: {
            if !(*top_node).children.is_null() && !(*(*top_node).children).next.is_null() {
            } else {
                __assert_fail(
                    b"top_node->children != NULL && top_node->children->next != NULL\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/viml/parser/expressions.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    1301 as ::core::ffi::c_uint,
                    b"_Bool viml_pexpr_handle_bop(const ParserState *const, ExprASTStack *const, ExprASTNode *const, ExprASTWantedNode *const, ExprASTError *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        (*bop_node).children = (*(*top_node).children).next;
        (*(*top_node).children).next = bop_node;
        '_c2rust_label_4: {
            if (*(*bop_node).children).next.is_null() {
            } else {
                __assert_fail(
                    b"bop_node->children->next == NULL\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/viml/parser/expressions.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    1312 as ::core::ffi::c_uint,
                    b"_Bool viml_pexpr_handle_bop(const ParserState *const, ExprASTStack *const, ExprASTNode *const, ExprASTWantedNode *const, ExprASTError *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if (*ast_stack).size == (*ast_stack).capacity {
            (*ast_stack).capacity = if (*ast_stack).capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*ast_stack).capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*ast_stack).items = (if (*ast_stack).capacity
                == ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*ast_stack).items
                    == &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                {
                    (*ast_stack).items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                            as *mut ::core::ffi::c_void,
                        (*ast_stack).items as *mut ::core::ffi::c_void,
                        (*ast_stack)
                            .size
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                }
            } else {
                if (*ast_stack).items
                    == &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                {
                    memcpy(
                        xmalloc(
                            (*ast_stack)
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                        ),
                        (*ast_stack).items as *const ::core::ffi::c_void,
                        (*ast_stack)
                            .size
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                } else {
                    xrealloc(
                        (*ast_stack).items as *mut ::core::ffi::c_void,
                        (*ast_stack)
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                }
            }) as *mut *mut *mut ExprASTNode;
        } else {
        };
        let c2rust_fresh31 = (*ast_stack).size;
        (*ast_stack).size = (*ast_stack).size.wrapping_add(1);
        let c2rust_lvalue_ptr_1 = &raw mut *(*ast_stack).items.offset(c2rust_fresh31 as isize);
        *c2rust_lvalue_ptr_1 = top_node_p;
        if (*ast_stack).size == (*ast_stack).capacity {
            (*ast_stack).capacity = if (*ast_stack).capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*ast_stack).capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*ast_stack).items = (if (*ast_stack).capacity
                == ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*ast_stack).items
                    == &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                {
                    (*ast_stack).items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                            as *mut ::core::ffi::c_void,
                        (*ast_stack).items as *mut ::core::ffi::c_void,
                        (*ast_stack)
                            .size
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                }
            } else {
                if (*ast_stack).items
                    == &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                {
                    memcpy(
                        xmalloc(
                            (*ast_stack)
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                        ),
                        (*ast_stack).items as *const ::core::ffi::c_void,
                        (*ast_stack)
                            .size
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                } else {
                    xrealloc(
                        (*ast_stack).items as *mut ::core::ffi::c_void,
                        (*ast_stack)
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                }
            }) as *mut *mut *mut ExprASTNode;
        } else {
        };
        let c2rust_fresh32 = (*ast_stack).size;
        (*ast_stack).size = (*ast_stack).size.wrapping_add(1);
        let c2rust_lvalue_ptr_2 = &raw mut *(*ast_stack).items.offset(c2rust_fresh32 as isize);
        *c2rust_lvalue_ptr_2 = &raw mut (*(*top_node).children).next;
        if (*ast_stack).size == (*ast_stack).capacity {
            (*ast_stack).capacity = if (*ast_stack).capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*ast_stack).capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*ast_stack).items = (if (*ast_stack).capacity
                == ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                    .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                            .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*ast_stack).items
                    == &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                {
                    (*ast_stack).items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                            as *mut ::core::ffi::c_void,
                        (*ast_stack).items as *mut ::core::ffi::c_void,
                        (*ast_stack)
                            .size
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                }
            } else {
                if (*ast_stack).items
                    == &raw mut (*ast_stack).init_array as *mut *mut *mut ExprASTNode
                {
                    memcpy(
                        xmalloc(
                            (*ast_stack)
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                        ),
                        (*ast_stack).items as *const ::core::ffi::c_void,
                        (*ast_stack)
                            .size
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                } else {
                    xrealloc(
                        (*ast_stack).items as *mut ::core::ffi::c_void,
                        (*ast_stack)
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    )
                }
            }) as *mut *mut *mut ExprASTNode;
        } else {
        };
        let c2rust_fresh33 = (*ast_stack).size;
        (*ast_stack).size = (*ast_stack).size.wrapping_add(1);
        let c2rust_lvalue_ptr_3 = &raw mut *(*ast_stack).items.offset(c2rust_fresh33 as isize);
        *c2rust_lvalue_ptr_3 = &raw mut (*(*bop_node).children).next;
        if (*bop_node).type_0 as ::core::ffi::c_uint
            == kExprNodeComparison as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            east_set_error(
                pstate,
                ast_err,
                gettext(b"E15: Operator is not associative: %.*s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                (*bop_node).start,
            );
            ret = false_0 != 0;
        }
    }
    *want_node_p = kENodeValue;
    return ret;
}
#[inline(always)]
unsafe extern "C" fn shifted_pos(pos: ParserPosition, shift: size_t) -> ParserPosition {
    return ParserPosition {
        line: pos.line,
        col: pos.col.wrapping_add(shift),
    };
}
#[inline(always)]
unsafe extern "C" fn recol_pos(pos: ParserPosition, new_col: size_t) -> ParserPosition {
    return ParserPosition {
        line: pos.line,
        col: new_col,
    };
}
#[inline(always)]
unsafe extern "C" fn east_set_error(
    pstate: *const ParserState,
    ret_ast_err: *mut ExprASTError,
    msg: *const ::core::ffi::c_char,
    start: ParserPosition,
) {
    if !(*ret_ast_err).msg.is_null() {
        return;
    }
    let pline: ParserLine = *(*pstate).reader.lines.items.offset(start.line as isize);
    (*ret_ast_err).msg = msg;
    (*ret_ast_err).arg_len = pline.size.wrapping_sub(start.col) as ::core::ffi::c_int;
    (*ret_ast_err).arg = if !pline.data.is_null() {
        pline.data.offset(start.col as isize)
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
#[inline(always)]
unsafe extern "C" fn pt_is_assignment(pt: ExprASTParseType) -> bool {
    return pt as ::core::ffi::c_uint
        == kEPTAssignment as ::core::ffi::c_int as ::core::ffi::c_uint
        || pt as ::core::ffi::c_uint
            == kEPTSingleAssignment as ::core::ffi::c_int as ::core::ffi::c_uint;
}
unsafe extern "C" fn parse_quoted_string(
    pstate: *mut ParserState,
    node: *mut ExprASTNode,
    token: LexExprToken,
    mut _ast_stack: *const ExprASTStack,
    is_invalid: bool,
) {
    let pline: ParserLine = *(*pstate)
        .reader
        .lines
        .items
        .offset(token.start.line as isize);
    let s: *const ::core::ffi::c_char = pline.data.offset(token.start.col as isize);
    let e: *const ::core::ffi::c_char = s
        .offset(token.len as isize)
        .offset(-(token.data.str.closed as ::core::ffi::c_int as isize));
    let mut p: *const ::core::ffi::c_char = s.offset(1 as ::core::ffi::c_int as isize);
    let is_double: bool = token.type_0 as ::core::ffi::c_uint
        == kExprLexDoubleQuotedString as ::core::ffi::c_int as ::core::ffi::c_uint;
    let mut size: size_t = token
        .len
        .wrapping_sub(token.data.str.closed as size_t)
        .wrapping_sub(1 as size_t);
    let mut shifts: C2Rust_Unnamed_35 = C2Rust_Unnamed_35 {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<StringShift>(),
        init_array: [StringShift {
            start: 0,
            orig_len: 0,
            act_len: 0,
            escape_not_known: false,
        }; 16],
    };
    shifts.capacity = ::core::mem::size_of::<[StringShift; 16]>()
        .wrapping_div(::core::mem::size_of::<StringShift>())
        .wrapping_div(
            (::core::mem::size_of::<[StringShift; 16]>()
                .wrapping_rem(::core::mem::size_of::<StringShift>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    shifts.size = 0 as size_t;
    shifts.items = &raw mut shifts.init_array as *mut StringShift;
    if !is_double {
        viml_parser_highlight(
            pstate,
            token.start,
            1 as size_t,
            if is_invalid as ::core::ffi::c_int != 0 {
                b"NvimInvalidSingleQuote\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"NvimSingleQuote\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        while p < e {
            let chunk_e: *const ::core::ffi::c_char = memchr(
                p as *const ::core::ffi::c_void,
                '\'' as ::core::ffi::c_int,
                e.offset_from(p) as size_t,
            ) as *const ::core::ffi::c_char;
            if chunk_e.is_null() {
                break;
            }
            size = size.wrapping_sub(1);
            p = chunk_e.offset(2 as ::core::ffi::c_int as isize);
            if !(*pstate).colors.is_null() {
                if shifts.size == shifts.capacity {
                    shifts.capacity = if shifts.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[StringShift; 16]>()
                            .wrapping_div(::core::mem::size_of::<StringShift>())
                            .wrapping_div(
                                (::core::mem::size_of::<[StringShift; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<StringShift>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        shifts.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[StringShift; 16]>()
                            .wrapping_div(::core::mem::size_of::<StringShift>())
                            .wrapping_div(
                                (::core::mem::size_of::<[StringShift; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<StringShift>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    shifts.items = (if shifts.capacity
                        == ::core::mem::size_of::<[StringShift; 16]>()
                            .wrapping_div(::core::mem::size_of::<StringShift>())
                            .wrapping_div(
                                (::core::mem::size_of::<[StringShift; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<StringShift>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if shifts.items == &raw mut shifts.init_array as *mut StringShift {
                            shifts.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut shifts.init_array as *mut StringShift
                                    as *mut ::core::ffi::c_void,
                                shifts.items as *mut ::core::ffi::c_void,
                                shifts
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<StringShift>()),
                            )
                        }
                    } else {
                        if shifts.items == &raw mut shifts.init_array as *mut StringShift {
                            memcpy(
                                xmalloc(
                                    shifts
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<StringShift>()),
                                ),
                                shifts.items as *const ::core::ffi::c_void,
                                shifts
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<StringShift>()),
                            )
                        } else {
                            xrealloc(
                                shifts.items as *mut ::core::ffi::c_void,
                                shifts
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<StringShift>()),
                            )
                        }
                    }) as *mut StringShift;
                } else {
                };
                let c2rust_fresh35 = shifts.size;
                shifts.size = shifts.size.wrapping_add(1);
                *shifts.items.offset(c2rust_fresh35 as isize) = StringShift {
                    start: token
                        .start
                        .col
                        .wrapping_add(chunk_e.offset_from(s) as size_t),
                    orig_len: 2 as size_t,
                    act_len: 1 as size_t,
                    escape_not_known: false,
                };
            }
        }
        (*node).data.str.size = size;
        if size == 0 as size_t {
            (*node).data.str.value = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            let mut v_p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*node).data.str.value = xmallocz(size) as *mut ::core::ffi::c_char;
            v_p = (*node).data.str.value;
            p = s.offset(1 as ::core::ffi::c_int as isize);
            while p < e {
                let chunk_e_0: *const ::core::ffi::c_char = memchr(
                    p as *const ::core::ffi::c_void,
                    '\'' as ::core::ffi::c_int,
                    e.offset_from(p) as size_t,
                )
                    as *const ::core::ffi::c_char;
                if chunk_e_0.is_null() {
                    memcpy(
                        v_p as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        e.offset_from(p) as size_t,
                    );
                    break;
                } else {
                    memcpy(
                        v_p as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        chunk_e_0.offset_from(p) as size_t,
                    );
                    v_p = v_p.offset(
                        (chunk_e_0.offset_from(p) as size_t).wrapping_add(1 as size_t) as isize,
                    );
                    *v_p.offset(-1 as ::core::ffi::c_int as isize) = '\'' as ::core::ffi::c_char;
                    p = chunk_e_0.offset(2 as ::core::ffi::c_int as isize);
                }
            }
        }
    } else {
        viml_parser_highlight(
            pstate,
            token.start,
            1 as size_t,
            if is_invalid as ::core::ffi::c_int != 0 {
                b"NvimInvalidDoubleQuote\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"NvimDoubleQuote\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        p = s.offset(1 as ::core::ffi::c_int as isize);
        while p < e {
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && p.offset(1 as ::core::ffi::c_int as isize) < e
            {
                p = p.offset(1);
                if p.offset(1 as ::core::ffi::c_int as isize) == e {
                    size = size.wrapping_sub(1);
                    break;
                } else {
                    match *p as ::core::ffi::c_int {
                        60 => {
                            size = size.wrapping_add(5 as size_t);
                        }
                        120 | 88 => {
                            size = size.wrapping_sub(1);
                            if ascii_isxdigit(
                                *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            ) {
                                size = size.wrapping_sub(1);
                                if p.offset(2 as ::core::ffi::c_int as isize) < e
                                    && ascii_isxdigit(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0
                                {
                                    size = size.wrapping_sub(1);
                                }
                            }
                        }
                        117 | 85 => {
                            let esc_start: *const ::core::ffi::c_char = p;
                            let mut n: size_t =
                                (if *p as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
                                    4 as ::core::ffi::c_int
                                } else {
                                    8 as ::core::ffi::c_int
                                }) as size_t;
                            let mut nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            p = p.offset(1);
                            while p.offset(1 as ::core::ffi::c_int as isize) < e
                                && {
                                    let c2rust_fresh36 = n;
                                    n = n.wrapping_sub(1);
                                    c2rust_fresh36 != 0
                                }
                                && ascii_isxdigit(*p.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                p = p.offset(1);
                                nr = (nr << 4 as ::core::ffi::c_int)
                                    + hex2nr(*p as ::core::ffi::c_int);
                            }
                            size = size.wrapping_sub(
                                (p.offset_from(
                                    esc_start.offset(-(1 as ::core::ffi::c_int as isize)),
                                ) - utf_char2len(nr) as isize)
                                    as size_t,
                            );
                            p = p.offset(-1);
                        }
                        48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 => {
                            size = size.wrapping_sub(1);
                            p = p.offset(1);
                            if *p as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                                && *p as ::core::ffi::c_int <= '7' as ::core::ffi::c_int
                            {
                                size = size.wrapping_sub(1);
                                p = p.offset(1);
                                if p < e
                                    && *p as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                                    && *p as ::core::ffi::c_int <= '7' as ::core::ffi::c_int
                                {
                                    size = size.wrapping_sub(1);
                                    p = p.offset(1);
                                }
                            }
                        }
                        _ => {
                            size = size.wrapping_sub(1);
                        }
                    }
                }
            }
            p = p.offset(1);
        }
        if size == 0 as size_t {
            (*node).data.str.value = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*node).data.str.size = 0 as size_t;
        } else {
            let mut v_p_0: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*node).data.str.value = xmalloc(size) as *mut ::core::ffi::c_char;
            v_p_0 = (*node).data.str.value;
            p = s.offset(1 as ::core::ffi::c_int as isize);
            while p < e {
                let chunk_e_1: *const ::core::ffi::c_char = memchr(
                    p as *const ::core::ffi::c_void,
                    '\\' as ::core::ffi::c_int,
                    e.offset_from(p) as size_t,
                )
                    as *const ::core::ffi::c_char;
                if chunk_e_1.is_null() {
                    memcpy(
                        v_p_0 as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        e.offset_from(p) as size_t,
                    );
                    v_p_0 = v_p_0.offset(e.offset_from(p) as isize);
                    break;
                } else {
                    memcpy(
                        v_p_0 as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        chunk_e_1.offset_from(p) as size_t,
                    );
                    v_p_0 = v_p_0.offset(chunk_e_1.offset_from(p) as size_t as isize);
                    p = chunk_e_1.offset(1 as ::core::ffi::c_int as isize);
                    if p == e {
                        let c2rust_fresh37 = v_p_0;
                        v_p_0 = v_p_0.offset(1);
                        *c2rust_fresh37 = '\\' as ::core::ffi::c_char;
                        break;
                    } else {
                        let mut is_unknown: bool = false_0 != 0;
                        let v_p_start: *const ::core::ffi::c_char = v_p_0;
                        match *p as ::core::ffi::c_int {
                            98 => {
                                let c2rust_fresh38 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh38 = '\u{8}' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            101 => {
                                let c2rust_fresh39 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh39 = '\u{1b}' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            102 => {
                                let c2rust_fresh40 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh40 = '\u{c}' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            110 => {
                                let c2rust_fresh41 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh41 = '\n' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            114 => {
                                let c2rust_fresh42 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh42 = '\r' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            116 => {
                                let c2rust_fresh43 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh43 = '\t' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            34 => {
                                let c2rust_fresh44 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh44 = '"' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            92 => {
                                let c2rust_fresh45 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh45 = '\\' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            88 | 120 | 117 | 85 => {
                                if p.offset(1 as ::core::ffi::c_int as isize) < e
                                    && ascii_isxdigit(*p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0
                                {
                                    let mut n_0: size_t = 0;
                                    let mut nr_0: ::core::ffi::c_int = 0;
                                    let mut is_hex: bool = *p as ::core::ffi::c_int
                                        == 'x' as ::core::ffi::c_int
                                        || *p as ::core::ffi::c_int == 'X' as ::core::ffi::c_int;
                                    if is_hex {
                                        n_0 = 2 as size_t;
                                    } else if *p as ::core::ffi::c_int == 'u' as ::core::ffi::c_int
                                    {
                                        n_0 = 4 as size_t;
                                    } else {
                                        n_0 = 8 as size_t;
                                    }
                                    nr_0 = 0 as ::core::ffi::c_int;
                                    while p.offset(1 as ::core::ffi::c_int as isize) < e
                                        && {
                                            let c2rust_fresh46 = n_0;
                                            n_0 = n_0.wrapping_sub(1);
                                            c2rust_fresh46 != 0
                                        }
                                        && ascii_isxdigit(
                                            *p.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        p = p.offset(1);
                                        nr_0 = (nr_0 << 4 as ::core::ffi::c_int)
                                            + hex2nr(*p as ::core::ffi::c_int);
                                    }
                                    p = p.offset(1);
                                    if is_hex {
                                        let c2rust_fresh47 = v_p_0;
                                        v_p_0 = v_p_0.offset(1);
                                        *c2rust_fresh47 = nr_0 as ::core::ffi::c_char;
                                    } else {
                                        v_p_0 = v_p_0.offset(utf_char2bytes(nr_0, v_p_0) as isize);
                                    }
                                } else {
                                    is_unknown = true_0 != 0;
                                    let c2rust_fresh48 = v_p_0;
                                    v_p_0 = v_p_0.offset(1);
                                    *c2rust_fresh48 = *p;
                                    p = p.offset(1);
                                }
                            }
                            48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 => {
                                let c2rust_fresh49 = p;
                                p = p.offset(1);
                                let mut ch: uint8_t = (*c2rust_fresh49 as ::core::ffi::c_int
                                    - '0' as ::core::ffi::c_int)
                                    as uint8_t;
                                if p < e
                                    && *p as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                                    && *p as ::core::ffi::c_int <= '7' as ::core::ffi::c_int
                                {
                                    let c2rust_fresh50 = p;
                                    p = p.offset(1);
                                    ch = (((ch as ::core::ffi::c_int) << 3 as ::core::ffi::c_int)
                                        + *c2rust_fresh50 as ::core::ffi::c_int
                                        - '0' as ::core::ffi::c_int)
                                        as uint8_t;
                                    if p < e
                                        && *p as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                                        && *p as ::core::ffi::c_int <= '7' as ::core::ffi::c_int
                                    {
                                        let c2rust_fresh51 = p;
                                        p = p.offset(1);
                                        ch = (((ch as ::core::ffi::c_int)
                                            << 3 as ::core::ffi::c_int)
                                            + *c2rust_fresh51 as ::core::ffi::c_int
                                            - '0' as ::core::ffi::c_int)
                                            as uint8_t;
                                    }
                                }
                                let c2rust_fresh52 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh52 = ch as ::core::ffi::c_char;
                            }
                            60 => {
                                let mut flags: ::core::ffi::c_int = FSK_KEYCODE
                                    as ::core::ffi::c_int
                                    | FSK_IN_STRING as ::core::ffi::c_int;
                                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                    != '*' as ::core::ffi::c_int
                                {
                                    flags |= FSK_SIMPLIFY as ::core::ffi::c_int;
                                }
                                let special_len: size_t = trans_special(
                                    &raw mut p,
                                    e.offset_from(p) as size_t,
                                    v_p_0,
                                    flags,
                                    false_0 != 0,
                                    ::core::ptr::null_mut::<bool>(),
                                )
                                    as size_t;
                                if special_len != 0 as size_t {
                                    v_p_0 = v_p_0.offset(special_len as isize);
                                } else {
                                    is_unknown = true_0 != 0;
                                    mb_copy_char(&raw mut p, &raw mut v_p_0);
                                }
                            }
                            _ => {
                                is_unknown = true_0 != 0;
                                mb_copy_char(&raw mut p, &raw mut v_p_0);
                            }
                        }
                        if !(*pstate).colors.is_null() {
                            if shifts.size == shifts.capacity {
                                shifts.capacity = if shifts.capacity << 1 as ::core::ffi::c_int
                                    > ::core::mem::size_of::<[StringShift; 16]>()
                                        .wrapping_div(::core::mem::size_of::<StringShift>())
                                        .wrapping_div(
                                            (::core::mem::size_of::<[StringShift; 16]>()
                                                .wrapping_rem(::core::mem::size_of::<StringShift>())
                                                == 0)
                                                as ::core::ffi::c_int
                                                as usize,
                                        ) {
                                    shifts.capacity << 1 as ::core::ffi::c_int
                                } else {
                                    ::core::mem::size_of::<[StringShift; 16]>()
                                        .wrapping_div(::core::mem::size_of::<StringShift>())
                                        .wrapping_div(
                                            (::core::mem::size_of::<[StringShift; 16]>()
                                                .wrapping_rem(::core::mem::size_of::<StringShift>())
                                                == 0)
                                                as ::core::ffi::c_int
                                                as size_t,
                                        )
                                };
                                shifts.items = (if shifts.capacity
                                    == ::core::mem::size_of::<[StringShift; 16]>()
                                        .wrapping_div(::core::mem::size_of::<StringShift>())
                                        .wrapping_div(
                                            (::core::mem::size_of::<[StringShift; 16]>()
                                                .wrapping_rem(::core::mem::size_of::<StringShift>())
                                                == 0)
                                                as ::core::ffi::c_int
                                                as usize,
                                        ) {
                                    if shifts.items
                                        == &raw mut shifts.init_array as *mut StringShift
                                    {
                                        shifts.items as *mut ::core::ffi::c_void
                                    } else {
                                        _memcpy_free(
                                            &raw mut shifts.init_array as *mut StringShift
                                                as *mut ::core::ffi::c_void,
                                            shifts.items as *mut ::core::ffi::c_void,
                                            shifts
                                                .size
                                                .wrapping_mul(::core::mem::size_of::<StringShift>()),
                                        )
                                    }
                                } else {
                                    if shifts.items
                                        == &raw mut shifts.init_array as *mut StringShift
                                    {
                                        memcpy(
                                            xmalloc(
                                                shifts
                                                    .capacity
                                                    .wrapping_mul(::core::mem::size_of::<StringShift>()),
                                            ),
                                            shifts.items as *const ::core::ffi::c_void,
                                            shifts
                                                .size
                                                .wrapping_mul(::core::mem::size_of::<StringShift>()),
                                        )
                                    } else {
                                        xrealloc(
                                            shifts.items as *mut ::core::ffi::c_void,
                                            shifts
                                                .capacity
                                                .wrapping_mul(::core::mem::size_of::<StringShift>()),
                                        )
                                    }
                                })
                                    as *mut StringShift;
                            } else {
                            };
                            let c2rust_fresh53 = shifts.size;
                            shifts.size = shifts.size.wrapping_add(1);
                            *shifts.items.offset(c2rust_fresh53 as isize) = StringShift {
                                start: token
                                    .start
                                    .col
                                    .wrapping_add(chunk_e_1.offset_from(s) as size_t),
                                orig_len: p.offset_from(chunk_e_1) as size_t,
                                act_len: v_p_0.offset_from(v_p_start as *mut ::core::ffi::c_char)
                                    as size_t,
                                escape_not_known: is_unknown,
                            };
                        }
                    }
                }
            }
            (*node).data.str.size = v_p_0.offset_from((*node).data.str.value) as size_t;
        }
    }
    if !(*pstate).colors.is_null() {
        let mut next_col: size_t = token.start.col.wrapping_add(1 as size_t);
        let body_str: *const ::core::ffi::c_char = if is_double as ::core::ffi::c_int != 0 {
            if is_invalid as ::core::ffi::c_int != 0 {
                b"NvimInvalidDoubleQuotedBody\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"NvimDoubleQuotedBody\0".as_ptr() as *const ::core::ffi::c_char
            }
        } else if is_invalid as ::core::ffi::c_int != 0 {
            b"NvimInvalidSingleQuotedBody\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"NvimSingleQuotedBody\0".as_ptr() as *const ::core::ffi::c_char
        };
        let esc_str: *const ::core::ffi::c_char = if is_double as ::core::ffi::c_int != 0 {
            if is_invalid as ::core::ffi::c_int != 0 {
                b"NvimInvalidDoubleQuotedEscape\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"NvimDoubleQuotedEscape\0".as_ptr() as *const ::core::ffi::c_char
            }
        } else if is_invalid as ::core::ffi::c_int != 0 {
            b"NvimInvalidSingleQuotedQuote\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"NvimSingleQuotedQuote\0".as_ptr() as *const ::core::ffi::c_char
        };
        let ukn_esc_str: *const ::core::ffi::c_char = if is_double as ::core::ffi::c_int != 0 {
            if is_invalid as ::core::ffi::c_int != 0 {
                b"NvimInvalidDoubleQuotedUnknownEscape\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"NvimDoubleQuotedUnknownEscape\0".as_ptr() as *const ::core::ffi::c_char
            }
        } else if is_invalid as ::core::ffi::c_int != 0 {
            b"NvimInvalidSingleQuotedUnknownEscape\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"NvimSingleQuotedUnknownEscape\0".as_ptr() as *const ::core::ffi::c_char
        };
        let mut i: size_t = 0 as size_t;
        while i < shifts.size {
            let cur_shift: StringShift = *shifts.items.offset(i as isize);
            if cur_shift.start > next_col {
                viml_parser_highlight(
                    pstate,
                    recol_pos(token.start, next_col),
                    cur_shift.start.wrapping_sub(next_col),
                    body_str,
                );
            }
            viml_parser_highlight(
                pstate,
                recol_pos(token.start, cur_shift.start),
                cur_shift.orig_len,
                if cur_shift.escape_not_known as ::core::ffi::c_int != 0 {
                    ukn_esc_str
                } else {
                    esc_str
                },
            );
            next_col = cur_shift.start.wrapping_add(cur_shift.orig_len);
            i = i.wrapping_add(1);
        }
        if next_col.wrapping_sub(token.start.col)
            < token.len.wrapping_sub(token.data.str.closed as size_t)
        {
            viml_parser_highlight(
                pstate,
                recol_pos(token.start, next_col),
                token
                    .start
                    .col
                    .wrapping_add(token.len)
                    .wrapping_sub(token.data.str.closed as size_t)
                    .wrapping_sub(next_col),
                body_str,
            );
        }
    }
    if token.data.str.closed {
        if is_double {
            viml_parser_highlight(
                pstate,
                shifted_pos(token.start, token.len.wrapping_sub(1 as size_t)),
                1 as size_t,
                if is_invalid as ::core::ffi::c_int != 0 {
                    b"NvimInvalidDoubleQuote\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"NvimDoubleQuote\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
        } else {
            viml_parser_highlight(
                pstate,
                shifted_pos(token.start, token.len.wrapping_sub(1 as size_t)),
                1 as size_t,
                if is_invalid as ::core::ffi::c_int != 0 {
                    b"NvimInvalidSingleQuote\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"NvimSingleQuote\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
        }
    }
    if shifts.items != &raw mut shifts.init_array as *mut StringShift {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut shifts.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
}
static want_node_to_lexer_flags: GlobalCell<[::core::ffi::c_int; 2]> = GlobalCell::new([
    kELFlagForbidScope as ::core::ffi::c_int,
    kELFlagIsNotCmp as ::core::ffi::c_int,
]);
static base_to_prefix_length: GlobalCell<[uint8_t; 17]> = GlobalCell::new([
    0,
    0,
    2 as uint8_t,
    0,
    0,
    0,
    0,
    0,
    1 as uint8_t,
    0,
    0 as uint8_t,
    0,
    0,
    0,
    0,
    0,
    2 as uint8_t,
]);
#[no_mangle]
pub unsafe extern "C" fn viml_pexpr_parse(
    pstate: *mut ParserState,
    flags: ::core::ffi::c_int,
) -> ExprAST {
    let mut can_be_ternary: bool = false;
    let mut is_subscript: bool = false;
    let mut i_0: size_t = 0;
    let mut eastnode_p: *const *mut ExprASTNode = ::core::ptr::null::<*mut ExprASTNode>();
    let mut eastnode_type: ExprASTNodeType = kExprNodeMissing;
    let mut eastnode_lvl: ExprOpLvl = kEOpLvlInvalid;
    let mut pline: ParserLine = ParserLine {
        data: ::core::ptr::null::<::core::ffi::c_char>(),
        size: 0,
        allocated: false,
    };
    let mut top_node_p: *mut *mut ExprASTNode = ::core::ptr::null_mut::<*mut ExprASTNode>();
    let mut cur_node: *mut ExprASTNode = ::core::ptr::null_mut::<ExprASTNode>();
    let mut want_value: bool = false;
    let mut node_is_key: bool = false;
    let mut is_single_assignment: bool = false;
    let mut cur_pt: ExprASTParseType = kEPTExpr;
    let mut ast: ExprAST = ExprAST {
        err: ExprASTError {
            msg: ::core::ptr::null::<::core::ffi::c_char>(),
            arg: ::core::ptr::null::<::core::ffi::c_char>(),
            arg_len: 0 as ::core::ffi::c_int,
        },
        root: ::core::ptr::null_mut::<ExprASTNode>(),
    };
    let mut ast_stack: ExprASTStack = ExprASTStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<*mut *mut ExprASTNode>(),
        init_array: [::core::ptr::null_mut::<*mut ExprASTNode>(); 16],
    };
    ast_stack.capacity = ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
        .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
        .wrapping_div(
            (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    ast_stack.size = 0 as size_t;
    ast_stack.items = &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode;
    if ast_stack.size == ast_stack.capacity {
        ast_stack.capacity = if ast_stack.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                .wrapping_div(
                    (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            ast_stack.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                .wrapping_div(
                    (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        ast_stack.items = (if ast_stack.capacity
            == ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                .wrapping_div(::core::mem::size_of::<*mut *mut ExprASTNode>())
                .wrapping_div(
                    (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                        .wrapping_rem(::core::mem::size_of::<*mut *mut ExprASTNode>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if ast_stack.items == &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode {
                ast_stack.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                        as *mut ::core::ffi::c_void,
                    ast_stack.items as *mut ::core::ffi::c_void,
                    ast_stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                )
            }
        } else {
            if ast_stack.items == &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode {
                memcpy(
                    xmalloc(
                        ast_stack
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                    ),
                    ast_stack.items as *const ::core::ffi::c_void,
                    ast_stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                )
            } else {
                xrealloc(
                    ast_stack.items as *mut ::core::ffi::c_void,
                    ast_stack
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<*mut *mut ExprASTNode>()),
                )
            }
        }) as *mut *mut *mut ExprASTNode;
    } else {
    };
    let c2rust_fresh4 = ast_stack.size;
    ast_stack.size = ast_stack.size.wrapping_add(1);
    let c2rust_lvalue_ptr = &raw mut *ast_stack.items.offset(c2rust_fresh4 as isize);
    *c2rust_lvalue_ptr = &raw mut ast.root;
    let mut want_node: ExprASTWantedNode = kENodeValue;
    let mut pt_stack: ExprASTParseTypeStack = ExprASTParseTypeStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<ExprASTParseType>(),
        init_array: [kEPTExpr; 4],
    };
    pt_stack.capacity = ::core::mem::size_of::<[ExprASTParseType; 4]>()
        .wrapping_div(::core::mem::size_of::<ExprASTParseType>())
        .wrapping_div(
            (::core::mem::size_of::<[ExprASTParseType; 4]>()
                .wrapping_rem(::core::mem::size_of::<ExprASTParseType>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    pt_stack.size = 0 as size_t;
    pt_stack.items = &raw mut pt_stack.init_array as *mut ExprASTParseType;
    if pt_stack.size == pt_stack.capacity {
        pt_stack.capacity = if pt_stack.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[ExprASTParseType; 4]>()
                .wrapping_div(::core::mem::size_of::<ExprASTParseType>())
                .wrapping_div(
                    (::core::mem::size_of::<[ExprASTParseType; 4]>()
                        .wrapping_rem(::core::mem::size_of::<ExprASTParseType>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            pt_stack.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[ExprASTParseType; 4]>()
                .wrapping_div(::core::mem::size_of::<ExprASTParseType>())
                .wrapping_div(
                    (::core::mem::size_of::<[ExprASTParseType; 4]>()
                        .wrapping_rem(::core::mem::size_of::<ExprASTParseType>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        pt_stack.items = (if pt_stack.capacity
            == ::core::mem::size_of::<[ExprASTParseType; 4]>()
                .wrapping_div(::core::mem::size_of::<ExprASTParseType>())
                .wrapping_div(
                    (::core::mem::size_of::<[ExprASTParseType; 4]>()
                        .wrapping_rem(::core::mem::size_of::<ExprASTParseType>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if pt_stack.items == &raw mut pt_stack.init_array as *mut ExprASTParseType {
                pt_stack.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut pt_stack.init_array as *mut ExprASTParseType
                        as *mut ::core::ffi::c_void,
                    pt_stack.items as *mut ::core::ffi::c_void,
                    pt_stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                )
            }
        } else {
            if pt_stack.items == &raw mut pt_stack.init_array as *mut ExprASTParseType {
                memcpy(
                    xmalloc(
                        pt_stack
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                    ),
                    pt_stack.items as *const ::core::ffi::c_void,
                    pt_stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                )
            } else {
                xrealloc(
                    pt_stack.items as *mut ::core::ffi::c_void,
                    pt_stack
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                )
            }
        }) as *mut ExprASTParseType;
    } else {
    };
    let c2rust_fresh5 = pt_stack.size;
    pt_stack.size = pt_stack.size.wrapping_add(1);
    *pt_stack.items.offset(c2rust_fresh5 as isize) = kEPTExpr;
    if flags & kExprFlagsParseLet as ::core::ffi::c_int != 0 {
        if pt_stack.size == pt_stack.capacity {
            pt_stack.capacity = if pt_stack.capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[ExprASTParseType; 4]>()
                    .wrapping_div(::core::mem::size_of::<ExprASTParseType>())
                    .wrapping_div(
                        (::core::mem::size_of::<[ExprASTParseType; 4]>()
                            .wrapping_rem(::core::mem::size_of::<ExprASTParseType>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                pt_stack.capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[ExprASTParseType; 4]>()
                    .wrapping_div(::core::mem::size_of::<ExprASTParseType>())
                    .wrapping_div(
                        (::core::mem::size_of::<[ExprASTParseType; 4]>()
                            .wrapping_rem(::core::mem::size_of::<ExprASTParseType>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            pt_stack.items = (if pt_stack.capacity
                == ::core::mem::size_of::<[ExprASTParseType; 4]>()
                    .wrapping_div(::core::mem::size_of::<ExprASTParseType>())
                    .wrapping_div(
                        (::core::mem::size_of::<[ExprASTParseType; 4]>()
                            .wrapping_rem(::core::mem::size_of::<ExprASTParseType>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if pt_stack.items == &raw mut pt_stack.init_array as *mut ExprASTParseType {
                    pt_stack.items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut pt_stack.init_array as *mut ExprASTParseType
                            as *mut ::core::ffi::c_void,
                        pt_stack.items as *mut ::core::ffi::c_void,
                        pt_stack
                            .size
                            .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                    )
                }
            } else {
                if pt_stack.items == &raw mut pt_stack.init_array as *mut ExprASTParseType {
                    memcpy(
                        xmalloc(
                            pt_stack
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                        ),
                        pt_stack.items as *const ::core::ffi::c_void,
                        pt_stack
                            .size
                            .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                    )
                } else {
                    xrealloc(
                        pt_stack.items as *mut ::core::ffi::c_void,
                        pt_stack
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                    )
                }
            }) as *mut ExprASTParseType;
        } else {
        };
        let c2rust_fresh6 = pt_stack.size;
        pt_stack.size = pt_stack.size.wrapping_add(1);
        *pt_stack.items.offset(c2rust_fresh6 as isize) = kEPTAssignment;
    }
    let mut prev_token: LexExprToken = LexExprToken {
        start: ParserPosition { line: 0, col: 0 },
        len: 0,
        type_0: kExprLexMissing,
        data: C2Rust_Unnamed_7 {
            cmp: C2Rust_Unnamed_19 {
                type_0: kExprCmpEqual,
                ccs: kCCStrategyUseOption,
                inv: false,
            },
        },
    };
    let mut highlighted_prev_spacing: bool = false_0 != 0;
    let mut lambda_node: *mut ExprASTNode = ::core::ptr::null_mut::<ExprASTNode>();
    let mut asgn_level: size_t = 0 as size_t;
    '_viml_pexpr_parse_end: loop {
        let is_concat_or_subscript: bool = want_node as ::core::ffi::c_uint
            == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
            && ast_stack.size > 1 as size_t
            && (***ast_stack.items.offset(
                ast_stack
                    .size
                    .wrapping_sub(1 as size_t)
                    .wrapping_sub(1 as size_t) as isize,
            ))
            .type_0 as ::core::ffi::c_uint
                == kExprNodeConcatOrSubscript as ::core::ffi::c_int as ::core::ffi::c_uint;
        let lexer_additional_flags: ::core::ffi::c_int = kELFlagPeek as ::core::ffi::c_int
            | (if flags & kExprFlagsDisallowEOC as ::core::ffi::c_int != 0 {
                kELFlagForbidEOC as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | (if want_node as ::core::ffi::c_uint
                == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                && (ast_stack.size == 1 as size_t
                    || (***ast_stack.items.offset(
                        ast_stack
                            .size
                            .wrapping_sub(1 as size_t)
                            .wrapping_sub(1 as size_t) as isize,
                    ))
                    .type_0 as ::core::ffi::c_uint
                        != kExprNodeConcat as ::core::ffi::c_int as ::core::ffi::c_uint
                        && (***ast_stack.items.offset(
                            ast_stack
                                .size
                                .wrapping_sub(1 as size_t)
                                .wrapping_sub(1 as size_t) as isize,
                        ))
                        .type_0 as ::core::ffi::c_uint
                            != kExprNodeConcatOrSubscript as ::core::ffi::c_int
                                as ::core::ffi::c_uint)
            {
                kELFlagAllowFloat as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
        let mut cur_token: LexExprToken = viml_pexpr_next_token(
            pstate,
            (*want_node_to_lexer_flags.ptr())[want_node as usize] | lexer_additional_flags,
        );
        if cur_token.type_0 as ::core::ffi::c_uint
            == kExprLexEOC as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        let mut tok_type: LexExprTokenType = cur_token.type_0;
        let token_invalid: bool = tok_type as ::core::ffi::c_uint
            == kExprLexInvalid as ::core::ffi::c_int as ::core::ffi::c_uint;
        let mut is_invalid: bool = token_invalid;
        '_viml_pexpr_parse_cycle_end: {
            's_6212: {
                's_4376: {
                    loop {
                        cur_token = viml_pexpr_next_token(
                            pstate,
                            (*want_node_to_lexer_flags.ptr())[want_node as usize]
                                | lexer_additional_flags,
                        );
                        if tok_type as ::core::ffi::c_uint
                            == kExprLexSpacing as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if is_invalid {
                                viml_parser_highlight(
                                    pstate,
                                    cur_token.start,
                                    cur_token.len,
                                    if is_invalid as ::core::ffi::c_int != 0 {
                                        b"NvimInvalidSpacing\0".as_ptr()
                                            as *const ::core::ffi::c_char
                                    } else {
                                        b"NvimSpacing\0".as_ptr() as *const ::core::ffi::c_char
                                    },
                                );
                            }
                            break '_viml_pexpr_parse_cycle_end;
                        } else {
                            if is_invalid as ::core::ffi::c_int != 0
                                && prev_token.type_0 as ::core::ffi::c_uint
                                    == kExprLexSpacing as ::core::ffi::c_int as ::core::ffi::c_uint
                                && !highlighted_prev_spacing
                            {
                                viml_parser_highlight(
                                    pstate,
                                    prev_token.start,
                                    prev_token.len,
                                    if is_invalid as ::core::ffi::c_int != 0 {
                                        b"NvimInvalidSpacing\0".as_ptr()
                                            as *const ::core::ffi::c_char
                                    } else {
                                        b"NvimSpacing\0".as_ptr() as *const ::core::ffi::c_char
                                    },
                                );
                                is_invalid = false_0 != 0;
                                highlighted_prev_spacing = true_0 != 0;
                            }
                            pline = *(*pstate)
                                .reader
                                .lines
                                .items
                                .offset(cur_token.start.line as isize);
                            top_node_p = *ast_stack.items.offset(
                                ast_stack
                                    .size
                                    .wrapping_sub(0 as size_t)
                                    .wrapping_sub(1 as size_t)
                                    as isize,
                            );
                            '_c2rust_label: {
                                if ast_stack.size >= 1 as size_t {
                                } else {
                                    __assert_fail(
                                        b"kv_size(ast_stack) >= 1\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/viml/parser/expressions.rs\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        1988 as ::core::ffi::c_uint,
                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            cur_node = ::core::ptr::null_mut::<ExprASTNode>();
                            want_value = want_node as ::core::ffi::c_uint
                                == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint;
                            '_c2rust_label_0: {
                                if want_value as ::core::ffi::c_int
                                    == (*top_node_p).is_null() as ::core::ffi::c_int
                                {
                                } else {
                                    __assert_fail(
                                        b"want_value == (*top_node_p == NULL)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/viml/parser/expressions.rs\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        1992 as ::core::ffi::c_uint,
                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            '_c2rust_label_1: {
                                if *ast_stack.items.offset(0 as ::core::ffi::c_int as isize)
                                    == &raw mut ast.root
                                {
                                } else {
                                    __assert_fail(
                                        b"kv_A(ast_stack, 0) == &ast.root\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/viml/parser/expressions.rs\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        1993 as ::core::ffi::c_uint,
                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            let mut i: size_t = 0 as size_t;
                            while i.wrapping_add(1 as size_t) < ast_stack.size {
                                let item_null: bool = want_value as ::core::ffi::c_int != 0
                                    && i.wrapping_add(2 as size_t) == ast_stack.size;
                                '_c2rust_label_2: {
                                    if &raw mut (***ast_stack.items.offset(i as isize)).children
                                        == *ast_stack
                                            .items
                                            .offset(i.wrapping_add(1 as size_t) as isize)
                                        && (if item_null as ::core::ffi::c_int != 0 {
                                            (***ast_stack.items.offset(i as isize))
                                                .children
                                                .is_null()
                                                as ::core::ffi::c_int
                                        } else {
                                            (*(***ast_stack.items.offset(i as isize)).children)
                                                .next
                                                .is_null()
                                                as ::core::ffi::c_int
                                        }) != 0
                                        || &raw mut (*(***ast_stack.items.offset(i as isize))
                                            .children)
                                            .next
                                            == *ast_stack
                                                .items
                                                .offset(i.wrapping_add(1 as size_t) as isize)
                                            && (if item_null as ::core::ffi::c_int != 0 {
                                                (*(***ast_stack.items.offset(i as isize)).children)
                                                    .next
                                                    .is_null()
                                                    as ::core::ffi::c_int
                                            } else {
                                                (*(*(***ast_stack.items.offset(i as isize))
                                                    .children)
                                                    .next)
                                                    .next
                                                    .is_null()
                                                    as ::core::ffi::c_int
                                            }) != 0
                                    {
                                    } else {
                                        __assert_fail(
                                            b"(&(*kv_A(ast_stack, i))->children == kv_A(ast_stack, i + 1) && (item_null ? (*kv_A(ast_stack, i))->children == NULL : (*kv_A(ast_stack, i))->children->next == NULL)) || ((&(*kv_A(ast_stack, i))->children->next == kv_A(ast_stack, i + 1)) && (item_null ? (*kv_A(ast_stack, i))->children->next == NULL : (*kv_A(ast_stack, i))->children->next->next == NULL))\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            b"src/nvim/viml/parser/expressions.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            2005 as ::core::ffi::c_uint,
                                            b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                i = i.wrapping_add(1);
                            }
                            node_is_key = is_concat_or_subscript as ::core::ffi::c_int != 0
                                && (if cur_token.type_0 as ::core::ffi::c_uint
                                    == kExprLexPlainIdentifier as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                {
                                    (!cur_token.data.var.autoload
                                        && cur_token.data.var.scope as ::core::ffi::c_uint
                                            == kExprVarScopeMissing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint)
                                        as ::core::ffi::c_int
                                } else {
                                    (cur_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexNumber as ::core::ffi::c_int
                                            as ::core::ffi::c_uint)
                                        as ::core::ffi::c_int
                                }) != 0
                                && prev_token.type_0 as ::core::ffi::c_uint
                                    != kExprLexSpacing as ::core::ffi::c_int as ::core::ffi::c_uint;
                            if is_concat_or_subscript as ::core::ffi::c_int != 0 && !node_is_key {
                                (***ast_stack.items.offset(
                                    ast_stack
                                        .size
                                        .wrapping_sub(1 as size_t)
                                        .wrapping_sub(1 as size_t)
                                        as isize,
                                ))
                                .type_0 = kExprNodeConcat;
                            }
                            is_single_assignment = *pt_stack.items.offset(
                                pt_stack
                                    .size
                                    .wrapping_sub(0 as size_t)
                                    .wrapping_sub(1 as size_t)
                                    as isize,
                            )
                                as ::core::ffi::c_uint
                                == kEPTSingleAssignment as ::core::ffi::c_int
                                    as ::core::ffi::c_uint;
                            match *pt_stack.items.offset(
                                pt_stack
                                    .size
                                    .wrapping_sub(0 as size_t)
                                    .wrapping_sub(1 as size_t)
                                    as isize,
                            ) as ::core::ffi::c_uint
                            {
                                1 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexComma as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexArrow as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || want_node as ::core::ffi::c_uint
                                            == kENodeValue as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            && !(cur_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexPlainIdentifier as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                && cur_token.data.var.scope as ::core::ffi::c_uint
                                                    == kExprVarScopeMissing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                && !cur_token.data.var.autoload)
                                            && tok_type as ::core::ffi::c_uint
                                                != kExprLexArrow as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                    {
                                        (*lambda_node).data.fig.type_guesses.allow_lambda =
                                            false_0 != 0;
                                        if !(*lambda_node).children.is_null()
                                            && (*(*lambda_node).children).type_0
                                                as ::core::ffi::c_uint
                                                == kExprNodeComma as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                        {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Expected lambda arguments list or arrow: %.*s\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                        } else {
                                            lambda_node = ::core::ptr::null_mut::<ExprASTNode>();
                                            pt_stack.size = pt_stack.size.wrapping_sub(1 as size_t);
                                        }
                                    }
                                }
                                3 | 2 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexBracket as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexPlainIdentifier as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && (tok_type as ::core::ffi::c_uint
                                            != kExprLexFigureBrace as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || cur_token.data.brc.closing as ::core::ffi::c_int
                                                != 0)
                                        && !(node_is_key as ::core::ffi::c_int != 0
                                            && tok_type as ::core::ffi::c_uint
                                                == kExprLexNumber as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint)
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexEnv as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexOption as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexRegister as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Expected value part of assignment lvalue: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        pt_stack.size = pt_stack.size.wrapping_sub(1 as size_t);
                                    } else if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexBracket as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && (tok_type as ::core::ffi::c_uint
                                            != kExprLexFigureBrace as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || cur_token.data.brc.closing as ::core::ffi::c_int
                                                != 0)
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexDot as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && (tok_type as ::core::ffi::c_uint
                                            != kExprLexComma as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || !is_single_assignment)
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexAssignment as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && !((tok_type as ::core::ffi::c_uint
                                            == kExprLexPlainIdentifier as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || tok_type as ::core::ffi::c_uint
                                                == kExprLexFigureBrace as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                && !cur_token.data.brc.closing)
                                            && prev_token.type_0 as ::core::ffi::c_uint
                                                != kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint)
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.size == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Expected assignment operator or subscript: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        pt_stack.size = pt_stack.size.wrapping_sub(1 as size_t);
                                    }
                                    '_c2rust_label_3: {
                                        if pt_stack.size != 0 {
                                        } else {
                                            __assert_fail(
                                                b"kv_size(pt_stack)\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"src/nvim/viml/parser/expressions.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                2107 as ::core::ffi::c_uint,
                                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                }
                                0 | _ => {}
                            }
                            '_c2rust_label_4: {
                                if pt_stack.size != 0 {
                                } else {
                                    __assert_fail(
                                        b"kv_size(pt_stack)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/viml/parser/expressions.rs\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        2110 as ::core::ffi::c_uint,
                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            cur_pt = *pt_stack.items.offset(
                                pt_stack
                                    .size
                                    .wrapping_sub(0 as size_t)
                                    .wrapping_sub(1 as size_t)
                                    as isize,
                            );
                            '_c2rust_label_5: {
                                if lambda_node.is_null()
                                    || cur_pt as ::core::ffi::c_uint
                                        == kEPTLambdaArguments as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                {
                                } else {
                                    __assert_fail(
                                        b"lambda_node == NULL || cur_pt == kEPTLambdaArguments\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/viml/parser/expressions.rs\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        2112 as ::core::ffi::c_uint,
                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            match tok_type as ::core::ffi::c_uint {
                                1 | 2 | 3 => {
                                    abort();
                                }
                                0 => {
                                    is_invalid = true_0 != 0;
                                    east_set_error(
                                        pstate,
                                        &raw mut ast.err,
                                        cur_token.data.err.msg,
                                        cur_token.start,
                                    );
                                    tok_type = cur_token.data.err.type_0;
                                }
                                18 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.size == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        '_c2rust_label_6: {
                                            if !(*top_node_p).is_null() {
                                            } else {
                                                __assert_fail(
                                                    b"*top_node_p != NULL\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2141 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeRegister);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).data.reg.name = cur_token.data.reg.name;
                                        *top_node_p = cur_node;
                                        want_node = kENodeOperator;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidRegister\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimRegister\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                9 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        cur_node = viml_pexpr_new_node(kExprNodeUnaryPlus);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        *top_node_p = cur_node;
                                        if ast_stack.size == ast_stack.capacity {
                                            ast_stack.capacity =
                                                if ast_stack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                                };
                                            ast_stack.items =
                                                (if ast_stack.capacity
                                                    == ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            ast_stack.items
                                                                as *const ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut *mut *mut ExprASTNode;
                                        } else {
                                        };
                                        let c2rust_fresh7 = ast_stack.size;
                                        ast_stack.size = ast_stack.size.wrapping_add(1);
                                        let c2rust_lvalue_ptr_0 = &raw mut *ast_stack
                                            .items
                                            .offset(c2rust_fresh7 as isize);
                                        *c2rust_lvalue_ptr_0 = &raw mut (*cur_node).children;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidUnaryPlus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimUnaryPlus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeBinaryPlus);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidBinaryPlus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimBinaryPlus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    }
                                    want_node = kENodeValue;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                10 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        cur_node = viml_pexpr_new_node(kExprNodeUnaryMinus);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        *top_node_p = cur_node;
                                        if ast_stack.size == ast_stack.capacity {
                                            ast_stack.capacity =
                                                if ast_stack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                                };
                                            ast_stack.items =
                                                (if ast_stack.capacity
                                                    == ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            ast_stack.items
                                                                as *const ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut *mut *mut ExprASTNode;
                                        } else {
                                        };
                                        let c2rust_fresh8 = ast_stack.size;
                                        ast_stack.size = ast_stack.size.wrapping_add(1);
                                        let c2rust_lvalue_ptr_1 = &raw mut *ast_stack
                                            .items
                                            .offset(c2rust_fresh8 as isize);
                                        *c2rust_lvalue_ptr_1 = &raw mut (*cur_node).children;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidUnaryMinus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimUnaryMinus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeBinaryMinus);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidBinaryMinus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimBinaryMinus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    }
                                    want_node = kENodeValue;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                6 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Unexpected or operator: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeOr);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    viml_parser_highlight(
                                        pstate,
                                        cur_token.start,
                                        cur_token.len,
                                        if is_invalid as ::core::ffi::c_int != 0 {
                                            b"NvimInvalidOr\0".as_ptr()
                                                as *const ::core::ffi::c_char
                                        } else {
                                            b"NvimOr\0".as_ptr() as *const ::core::ffi::c_char
                                        },
                                    );
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &raw mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                7 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Unexpected and operator: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeAnd);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    viml_parser_highlight(
                                        pstate,
                                        cur_token.start,
                                        cur_token.len,
                                        if is_invalid as ::core::ffi::c_int != 0 {
                                            b"NvimInvalidAnd\0".as_ptr()
                                                as *const ::core::ffi::c_char
                                        } else {
                                            b"NvimAnd\0".as_ptr() as *const ::core::ffi::c_char
                                        },
                                    );
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &raw mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                12 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Unexpected multiplication-like operator: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    match cur_token.data.mul.type_0 as ::core::ffi::c_uint {
                                        0 => {
                                            cur_node = viml_pexpr_new_node(kExprNodeMultiplication);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidMultiplication\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimMultiplication\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        1 => {
                                            cur_node = viml_pexpr_new_node(kExprNodeDivision);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidDivision\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimDivision\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        2 => {
                                            cur_node = viml_pexpr_new_node(kExprNodeMod);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidMod\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimMod\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        _ => {}
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &raw mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                17 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.size == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        '_c2rust_label_7: {
                                            if !(*top_node_p).is_null() {
                                            } else {
                                                __assert_fail(
                                                    b"*top_node_p != NULL\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2182 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeOption);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        if cur_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexInvalid as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            '_c2rust_label_8: {
                                                if cur_token.len == 1 as size_t
                                                    || cur_token.len == 3 as size_t
                                                        && *pline.data.offset(
                                                            cur_token
                                                                .start
                                                                .col
                                                                .wrapping_add(2 as size_t)
                                                                as isize,
                                                        )
                                                            as ::core::ffi::c_int
                                                            == ':' as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"cur_token.len == 1 || (cur_token.len == 3 && pline.data[cur_token.start.col + 2] == ':')\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2188 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            (*cur_node).data.opt.ident = pline
                                                .data
                                                .offset(cur_token.start.col as isize)
                                                .offset(cur_token.len as isize);
                                            (*cur_node).data.opt.ident_len = 0 as size_t;
                                            (*cur_node).data.opt.scope = (if cur_token.len
                                                == 3 as size_t
                                            {
                                                *pline.data.offset(
                                                    cur_token.start.col.wrapping_add(1 as size_t)
                                                        as isize,
                                                )
                                                    as ExprOptScope
                                                    as ::core::ffi::c_uint
                                            } else {
                                                kExprOptScopeUnspecified as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            })
                                                as ExprOptScope;
                                        } else {
                                            (*cur_node).data.opt.ident = cur_token.data.opt.name;
                                            (*cur_node).data.opt.ident_len = cur_token.data.opt.len;
                                            (*cur_node).data.opt.scope = cur_token.data.opt.scope;
                                        }
                                        *top_node_p = cur_node;
                                        want_node = kENodeOperator;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            1 as size_t,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidOptionSigil\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimOptionSigil\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        let scope_shift: size_t = (if cur_token.data.opt.scope
                                            as ::core::ffi::c_uint
                                            == kExprOptScopeUnspecified as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            0 as ::core::ffi::c_int
                                        } else {
                                            2 as ::core::ffi::c_int
                                        })
                                            as size_t;
                                        if scope_shift != 0 {
                                            viml_parser_highlight(
                                                pstate,
                                                shifted_pos(cur_token.start, 1 as size_t),
                                                1 as size_t,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidOptionScope\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimOptionScope\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                shifted_pos(cur_token.start, 2 as size_t),
                                                1 as size_t,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidOptionScopeDelimiter\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimOptionScopeDelimiter\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        viml_parser_highlight(
                                            pstate,
                                            shifted_pos(
                                                cur_token.start,
                                                scope_shift.wrapping_add(1 as size_t),
                                            ),
                                            cur_token.len.wrapping_sub(
                                                scope_shift.wrapping_add(1 as size_t),
                                            ),
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidOptionName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimOptionName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                19 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.size == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        '_c2rust_label_9: {
                                            if !(*top_node_p).is_null() {
                                            } else {
                                                __assert_fail(
                                                    b"*top_node_p != NULL\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2218 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeEnvironment);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).data.env.ident = pline
                                            .data
                                            .offset(cur_token.start.col as isize)
                                            .offset(1 as ::core::ffi::c_int as isize);
                                        (*cur_node).data.env.ident_len =
                                            cur_token.len.wrapping_sub(1 as size_t);
                                        if (*cur_node).data.env.ident_len == 0 as size_t {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Environment variable name missing\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                        }
                                        *top_node_p = cur_node;
                                        want_node = kENodeOperator;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            1 as size_t,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidEnvironmentSigil\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimEnvironmentSigil\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        viml_parser_highlight(
                                            pstate,
                                            shifted_pos(cur_token.start, 1 as size_t),
                                            cur_token.len.wrapping_sub(1 as size_t),
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidEnvironmentName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimEnvironmentName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                13 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.size == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        '_c2rust_label_10: {
                                            if !(*top_node_p).is_null() {
                                            } else {
                                                __assert_fail(
                                                    b"*top_node_p != NULL\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2235 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeNot);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        *top_node_p = cur_node;
                                        if ast_stack.size == ast_stack.capacity {
                                            ast_stack.capacity =
                                                if ast_stack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                                };
                                            ast_stack.items =
                                                (if ast_stack.capacity
                                                    == ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            ast_stack.items
                                                                as *const ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut *mut *mut ExprASTNode;
                                        } else {
                                        };
                                        let c2rust_fresh9 = ast_stack.size;
                                        ast_stack.size = ast_stack.size.wrapping_add(1);
                                        let c2rust_lvalue_ptr_2 = &raw mut *ast_stack
                                            .items
                                            .offset(c2rust_fresh9 as isize);
                                        *c2rust_lvalue_ptr_2 = &raw mut (*cur_node).children;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidNot\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimNot\0".as_ptr() as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                8 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Expected value, got comparison operator: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeComparison);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    if cur_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexInvalid as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).data.cmp.ccs = kCCStrategyUseOption;
                                        (*cur_node).data.cmp.type_0 = kExprCmpEqual;
                                        (*cur_node).data.cmp.inv = false_0 != 0;
                                    } else {
                                        (*cur_node).data.cmp.ccs = cur_token.data.cmp.ccs;
                                        (*cur_node).data.cmp.type_0 = cur_token.data.cmp.type_0;
                                        (*cur_node).data.cmp.inv = cur_token.data.cmp.inv;
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &raw mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    if cur_token.data.cmp.ccs as ::core::ffi::c_uint
                                        != kCCStrategyUseOption as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len.wrapping_sub(1 as size_t),
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidComparison\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimComparison\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        viml_parser_highlight(
                                            pstate,
                                            shifted_pos(
                                                cur_token.start,
                                                cur_token.len.wrapping_sub(1 as size_t),
                                            ),
                                            1 as size_t,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidComparisonModifier\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimComparisonModifier\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    } else {
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidComparison\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimComparison\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    }
                                    want_node = kENodeValue;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                24 => {
                                    '_c2rust_label_11: {
                                        if !(want_node as ::core::ffi::c_uint
                                            == kENodeValue as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            && cur_pt as ::core::ffi::c_uint
                                                == kEPTLambdaArguments as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint)
                                        {
                                        } else {
                                            __assert_fail(
                                                b"!(want_node == kENodeValue && cur_pt == kEPTLambdaArguments)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                b"src/nvim/viml/parser/expressions.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                2266 as ::core::ffi::c_uint,
                                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Expected value, got comma: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        *top_node_p = cur_node;
                                        want_node = kENodeOperator;
                                    }
                                    if cur_pt as ::core::ffi::c_uint
                                        == kEPTLambdaArguments as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        '_c2rust_label_12: {
                                            if !lambda_node.is_null() {
                                            } else {
                                                __assert_fail(
                                                    b"lambda_node != NULL\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2277 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        '_c2rust_label_13: {
                                            if (*lambda_node).data.fig.type_guesses.allow_lambda {
                                            } else {
                                                __assert_fail(
                                                    b"lambda_node->data.fig.type_guesses.allow_lambda\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2278 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        let node_: *mut ExprASTNode = lambda_node;
                                        '_c2rust_label_14: {
                                            if (*node_).type_0 as ::core::ffi::c_uint
                                                == kExprNodeUnknownFigure as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || (*node_).type_0 as ::core::ffi::c_uint
                                                    == kExprNodeLambda as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"node_->type == kExprNodeUnknownFigure || node_->type == kExprNodeLambda\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2279 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        (*node_).type_0 = kExprNodeLambda;
                                        if !(*pstate).colors.is_null() {
                                            (*(*(*pstate).colors).items.offset(
                                                (*node_).data.fig.opening_hl_idx as isize,
                                            ))
                                            .group = if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidLambda\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimLambda\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            };
                                        }
                                    }
                                    's_2222: {
                                        '_viml_pexpr_parse_invalid_comma: {
                                            if ast_stack.size >= 2 as size_t {
                                                i_0 = 1 as size_t;
                                                loop {
                                                    if i_0 >= ast_stack.size {
                                                        break 's_2222;
                                                    }
                                                    eastnode_p = *ast_stack.items.offset(
                                                        ast_stack
                                                            .size
                                                            .wrapping_sub(i_0)
                                                            .wrapping_sub(1 as size_t)
                                                            as isize,
                                                    )
                                                        as *const *mut ExprASTNode;
                                                    eastnode_type = (**eastnode_p).type_0;
                                                    eastnode_lvl = node_lvl(**eastnode_p);
                                                    if eastnode_type as ::core::ffi::c_uint
                                                        == kExprNodeLambda as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        '_c2rust_label_15: {
                                                            if cur_pt as ::core::ffi::c_uint
                                                                == kEPTLambdaArguments
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                && want_node as ::core::ffi::c_uint
                                                                    == kENodeOperator
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                            {
                                                            } else {
                                                                __assert_fail(
                                                                    b"cur_pt == kEPTLambdaArguments && want_node == kENodeOperator\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                    2291 as ::core::ffi::c_uint,
                                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                );
                                                            }
                                                        };
                                                        break 's_2222;
                                                    } else {
                                                        if eastnode_type as ::core::ffi::c_uint
                                                            == kExprNodeDictLiteral
                                                                as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                            || eastnode_type as ::core::ffi::c_uint
                                                                == kExprNodeListLiteral
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            || eastnode_type as ::core::ffi::c_uint
                                                                == kExprNodeCall
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                        {
                                                            break 's_2222;
                                                        }
                                                        if !(eastnode_type as ::core::ffi::c_uint
                                                            == kExprNodeComma as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                            || eastnode_type as ::core::ffi::c_uint
                                                                == kExprNodeColon
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            || eastnode_lvl as ::core::ffi::c_uint
                                                                > kEOpLvlComma as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint)
                                                        {
                                                            break '_viml_pexpr_parse_invalid_comma;
                                                        }
                                                        if i_0
                                                            == ast_stack
                                                                .size
                                                                .wrapping_sub(1 as size_t)
                                                        {
                                                            break '_viml_pexpr_parse_invalid_comma;
                                                        }
                                                        i_0 = i_0.wrapping_add(1);
                                                    }
                                                }
                                            }
                                        }
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Comma outside of call, lambda or literal: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeComma);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &raw mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    viml_parser_highlight(
                                        pstate,
                                        cur_token.start,
                                        cur_token.len,
                                        if is_invalid as ::core::ffi::c_int != 0 {
                                            b"NvimInvalidComma\0".as_ptr()
                                                as *const ::core::ffi::c_char
                                        } else {
                                            b"NvimComma\0".as_ptr() as *const ::core::ffi::c_char
                                        },
                                    );
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                5 => {
                                    let mut is_ternary: bool = false_0 != 0;
                                    's_2937: {
                                        '_viml_pexpr_parse_valid_colon: {
                                            '_viml_pexpr_parse_invalid_colon: {
                                                if ast_stack.size >= 2 as size_t {
                                                    can_be_ternary = true_0 != 0;
                                                    is_subscript = false_0 != 0;
                                                    let mut i_1: size_t = 1 as size_t;
                                                    while i_1 < ast_stack.size {
                                                        let eastnode_p_0: *const *mut ExprASTNode =
                                                            *ast_stack.items.offset(
                                                                ast_stack
                                                                    .size
                                                                    .wrapping_sub(i_1)
                                                                    .wrapping_sub(1 as size_t)
                                                                    as isize,
                                                            )
                                                                as *const *mut ExprASTNode;
                                                        let eastnode_type_0: ExprASTNodeType =
                                                            (**eastnode_p_0).type_0;
                                                        let eastnode_lvl_0: ExprOpLvl =
                                                            node_lvl(**eastnode_p_0);
                                                        if can_be_ternary as ::core::ffi::c_int != 0
                                                            && eastnode_type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprNodeTernaryValue
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            && !(**eastnode_p_0).data.ter.got_colon
                                                        {
                                                            ast_stack.size =
                                                                ast_stack.size.wrapping_sub(i_1);
                                                            (**eastnode_p_0).start =
                                                                cur_token.start;
                                                            (**eastnode_p_0).len = cur_token.len;
                                                            if prev_token.type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprLexSpacing
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                (**eastnode_p_0).start =
                                                                    prev_token.start;
                                                                (**eastnode_p_0).len =
                                                                    (**eastnode_p_0)
                                                                        .len
                                                                        .wrapping_add(
                                                                            prev_token.len,
                                                                        );
                                                            }
                                                            is_ternary = true_0 != 0;
                                                            (**eastnode_p_0).data.ter.got_colon =
                                                                true_0 != 0;
                                                            if want_node as ::core::ffi::c_uint
                                                                == kENodeValue as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                is_invalid = true_0 != 0;
                                                                east_set_error(
                                                                    pstate,
                                                                    &raw mut ast.err,
                                                                    gettext(
                                                                        b"E15: Expected value, got colon: %.*s\0".as_ptr()
                                                                            as *const ::core::ffi::c_char,
                                                                    ),
                                                                    cur_token.start,
                                                                );
                                                                *top_node_p = viml_pexpr_new_node(
                                                                    kExprNodeMissing,
                                                                );
                                                                (**top_node_p).start =
                                                                    cur_token.start;
                                                                (**top_node_p).len = cur_token.len;
                                                                if prev_token.type_0
                                                                    as ::core::ffi::c_uint
                                                                    == kExprLexSpacing
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                {
                                                                    (**top_node_p).start =
                                                                        prev_token.start;
                                                                    (**top_node_p).len =
                                                                        (**top_node_p)
                                                                            .len
                                                                            .wrapping_add(
                                                                                prev_token.len,
                                                                            );
                                                                }
                                                                (**top_node_p).len = 0 as size_t;
                                                                want_node = kENodeOperator;
                                                            }
                                                            '_c2rust_label_16: {
                                                                if !(**eastnode_p_0)
                                                                    .children
                                                                    .is_null()
                                                                {
                                                                } else {
                                                                    __assert_fail(
                                                                        b"(*eastnode_p)->children != NULL\0".as_ptr()
                                                                            as *const ::core::ffi::c_char,
                                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                        2342 as ::core::ffi::c_uint,
                                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                    );
                                                                }
                                                            };
                                                            '_c2rust_label_17: {
                                                                if (*(**eastnode_p_0).children)
                                                                    .next
                                                                    .is_null()
                                                                {
                                                                } else {
                                                                    __assert_fail(
                                                                        b"(*eastnode_p)->children->next == NULL\0".as_ptr()
                                                                            as *const ::core::ffi::c_char,
                                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                        2343 as ::core::ffi::c_uint,
                                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                    );
                                                                }
                                                            };
                                                            if ast_stack.size == ast_stack.capacity
                                                            {
                                                                ast_stack.capacity = if ast_stack.capacity
                                                                    << 1 as ::core::ffi::c_int
                                                                    > ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                                                        .wrapping_div(
                                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                        )
                                                                        .wrapping_div(
                                                                            (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                                                                .wrapping_rem(
                                                                                    ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                                ) == 0) as ::core::ffi::c_int as usize,
                                                                        )
                                                                {
                                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                                } else {
                                                                    ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                                                        .wrapping_div(
                                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                        )
                                                                        .wrapping_div(
                                                                            (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                                                                .wrapping_rem(
                                                                                    ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                                ) == 0) as ::core::ffi::c_int as size_t,
                                                                        )
                                                                };
                                                                ast_stack.items = (if ast_stack.capacity
                                                                    == ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                                                        .wrapping_div(
                                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                        )
                                                                        .wrapping_div(
                                                                            (::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                                                                .wrapping_rem(
                                                                                    ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                                ) == 0) as ::core::ffi::c_int as usize,
                                                                        )
                                                                {
                                                                    if ast_stack.items
                                                                        == &raw mut ast_stack.init_array
                                                                            as *mut *mut *mut ExprASTNode
                                                                    {
                                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                                    } else {
                                                                        _memcpy_free(
                                                                            &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                                                                                as *mut ::core::ffi::c_void,
                                                                            ast_stack.items as *mut ::core::ffi::c_void,
                                                                            ast_stack
                                                                                .size
                                                                                .wrapping_mul(
                                                                                    ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                                ),
                                                                        )
                                                                    }
                                                                } else {
                                                                    if ast_stack.items
                                                                        == &raw mut ast_stack.init_array
                                                                            as *mut *mut *mut ExprASTNode
                                                                    {
                                                                        memcpy(
                                                                            xmalloc(
                                                                                ast_stack
                                                                                    .capacity
                                                                                    .wrapping_mul(
                                                                                        ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                                    ),
                                                                            ),
                                                                            ast_stack.items as *const ::core::ffi::c_void,
                                                                            ast_stack
                                                                                .size
                                                                                .wrapping_mul(
                                                                                    ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                                ),
                                                                        )
                                                                    } else {
                                                                        xrealloc(
                                                                            ast_stack.items as *mut ::core::ffi::c_void,
                                                                            ast_stack
                                                                                .capacity
                                                                                .wrapping_mul(
                                                                                    ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                                ),
                                                                        )
                                                                    }
                                                                }) as *mut *mut *mut ExprASTNode;
                                                            } else {
                                                            };
                                                            let c2rust_fresh10 = ast_stack.size;
                                                            ast_stack.size =
                                                                ast_stack.size.wrapping_add(1);
                                                            let c2rust_lvalue_ptr_3 =
                                                                &raw mut *ast_stack.items.offset(
                                                                    c2rust_fresh10 as isize,
                                                                );
                                                            *c2rust_lvalue_ptr_3 =
                                                                &raw mut (*(**eastnode_p_0)
                                                                    .children)
                                                                    .next;
                                                            break;
                                                        } else if eastnode_type_0
                                                            as ::core::ffi::c_uint
                                                            == kExprNodeUnknownFigure
                                                                as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                        {
                                                            let node__0: *mut ExprASTNode =
                                                                *eastnode_p_0;
                                                            '_c2rust_label_18: {
                                                                if (*node__0).type_0
                                                                    as ::core::ffi::c_uint
                                                                    == kExprNodeUnknownFigure
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                    || (*node__0).type_0
                                                                        as ::core::ffi::c_uint
                                                                        == kExprNodeDictLiteral
                                                                            as ::core::ffi::c_int
                                                                            as ::core::ffi::c_uint
                                                                {
                                                                } else {
                                                                    __assert_fail(
                                                                        b"node_->type == kExprNodeUnknownFigure || node_->type == kExprNodeDictLiteral\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                        2347 as ::core::ffi::c_uint,
                                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                    );
                                                                }
                                                            };
                                                            (*node__0).type_0 =
                                                                kExprNodeDictLiteral;
                                                            if !(*pstate).colors.is_null() {
                                                                (*(*(*pstate).colors)
                                                                    .items
                                                                    .offset(
                                                                        (*node__0)
                                                                            .data
                                                                            .fig
                                                                            .opening_hl_idx
                                                                            as isize,
                                                                    ))
                                                                .group = if is_invalid
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                };
                                                            }
                                                            break;
                                                        } else {
                                                            if eastnode_type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprNodeDictLiteral
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                break;
                                                            }
                                                            if eastnode_type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprNodeSubscript
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                is_subscript = true_0 != 0;
                                                                '_c2rust_label_19: {
                                                                    if !is_ternary {
                                                                    } else {
                                                                        __assert_fail(
                                                                            b"!is_ternary\0".as_ptr() as *const ::core::ffi::c_char,
                                                                            b"src/nvim/viml/parser/expressions.rs\0"
                                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                                            2354 as ::core::ffi::c_uint,
                                                                            b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                                        );
                                                                    }
                                                                };
                                                                break;
                                                            } else {
                                                                if eastnode_type_0
                                                                    as ::core::ffi::c_uint
                                                                    == kExprNodeColon
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                {
                                                                    break '_viml_pexpr_parse_invalid_colon;
                                                                }
                                                                if (eastnode_lvl_0
                                                                    as ::core::ffi::c_uint)
                                                                    < kEOpLvlTernaryValue
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                {
                                                                    if (eastnode_lvl_0
                                                                        as ::core::ffi::c_uint)
                                                                        < kEOpLvlComma
                                                                            as ::core::ffi::c_int
                                                                            as ::core::ffi::c_uint
                                                                    {
                                                                        break '_viml_pexpr_parse_invalid_colon;
                                                                    }
                                                                    can_be_ternary = false_0 != 0;
                                                                }
                                                                if i_1
                                                                    == ast_stack
                                                                        .size
                                                                        .wrapping_sub(1 as size_t)
                                                                {
                                                                    break '_viml_pexpr_parse_invalid_colon;
                                                                }
                                                                i_1 = i_1.wrapping_add(1);
                                                            }
                                                        }
                                                    }
                                                    if is_subscript {
                                                        '_c2rust_label_20: {
                                                            if ast_stack.size > 1 as size_t {
                                                            } else {
                                                                __assert_fail(
                                                                    b"kv_size(ast_stack) > 1\0".as_ptr()
                                                                        as *const ::core::ffi::c_char,
                                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                    2370 as ::core::ffi::c_uint,
                                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                );
                                                            }
                                                        };
                                                        if want_node as ::core::ffi::c_uint
                                                            == kENodeValue as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                            && (***ast_stack.items.offset(
                                                                ast_stack
                                                                    .size
                                                                    .wrapping_sub(1 as size_t)
                                                                    .wrapping_sub(1 as size_t)
                                                                    as isize,
                                                            ))
                                                            .type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprNodeSubscript
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                        {
                                                            *top_node_p = viml_pexpr_new_node(
                                                                kExprNodeMissing,
                                                            );
                                                            (**top_node_p).start = cur_token.start;
                                                            (**top_node_p).len = cur_token.len;
                                                            if prev_token.type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprLexSpacing
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                (**top_node_p).start =
                                                                    prev_token.start;
                                                                (**top_node_p).len = (**top_node_p)
                                                                    .len
                                                                    .wrapping_add(prev_token.len);
                                                            }
                                                            (**top_node_p).len = 0 as size_t;
                                                            want_node = kENodeOperator;
                                                        } else if want_node as ::core::ffi::c_uint
                                                            == kENodeValue as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                        {
                                                            is_invalid = true_0 != 0;
                                                            east_set_error(
                                                                pstate,
                                                                &raw mut ast.err,
                                                                gettext(
                                                                    b"E15: Expected value, got colon: %.*s\0".as_ptr()
                                                                        as *const ::core::ffi::c_char,
                                                                ),
                                                                cur_token.start,
                                                            );
                                                            *top_node_p = viml_pexpr_new_node(
                                                                kExprNodeMissing,
                                                            );
                                                            (**top_node_p).start = cur_token.start;
                                                            (**top_node_p).len = cur_token.len;
                                                            if prev_token.type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprLexSpacing
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                (**top_node_p).start =
                                                                    prev_token.start;
                                                                (**top_node_p).len = (**top_node_p)
                                                                    .len
                                                                    .wrapping_add(prev_token.len);
                                                            }
                                                            (**top_node_p).len = 0 as size_t;
                                                            want_node = kENodeOperator;
                                                        }
                                                        cur_node =
                                                            viml_pexpr_new_node(kExprNodeColon);
                                                        (*cur_node).start = cur_token.start;
                                                        (*cur_node).len = cur_token.len;
                                                        if prev_token.type_0 as ::core::ffi::c_uint
                                                            == kExprLexSpacing as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                        {
                                                            (*cur_node).start = prev_token.start;
                                                            (*cur_node).len = (*cur_node)
                                                                .len
                                                                .wrapping_add(prev_token.len);
                                                        }
                                                        is_invalid = is_invalid
                                                            as ::core::ffi::c_int
                                                            | !viml_pexpr_handle_bop(
                                                                pstate,
                                                                &raw mut ast_stack,
                                                                cur_node,
                                                                &raw mut want_node,
                                                                &raw mut ast.err,
                                                            )
                                                                as ::core::ffi::c_int
                                                            != 0;
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidSubscriptColon\0"
                                                                    .as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimSubscriptColon\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_2937;
                                                    } else {
                                                        break '_viml_pexpr_parse_valid_colon;
                                                    }
                                                }
                                            }
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Colon outside of dictionary or ternary operator: %.*s\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                        }
                                        if want_node as ::core::ffi::c_uint
                                            == kENodeValue as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Expected value, got colon: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                            *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                            (**top_node_p).start = cur_token.start;
                                            (**top_node_p).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (**top_node_p).start = prev_token.start;
                                                (**top_node_p).len =
                                                    (**top_node_p).len.wrapping_add(prev_token.len);
                                            }
                                            (**top_node_p).len = 0 as size_t;
                                            want_node = kENodeOperator;
                                        }
                                        if is_ternary {
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidTernaryColon\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimTernaryColon\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        } else {
                                            cur_node = viml_pexpr_new_node(kExprNodeColon);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            is_invalid = is_invalid as ::core::ffi::c_int
                                                | !viml_pexpr_handle_bop(
                                                    pstate,
                                                    &raw mut ast_stack,
                                                    cur_node,
                                                    &raw mut want_node,
                                                    &raw mut ast.err,
                                                )
                                                    as ::core::ffi::c_int
                                                != 0;
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidColon\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimColon\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                    }
                                    want_node = kENodeValue;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                21 => {
                                    if cur_token.data.brc.closing {
                                        let mut new_top_node: *mut ExprASTNode =
                                            ::core::ptr::null_mut::<ExprASTNode>();
                                        let mut new_top_node_p: *mut *mut ExprASTNode =
                                            ::core::ptr::null_mut::<*mut ExprASTNode>();
                                        ast_stack.size = ast_stack.size.wrapping_sub(1 as size_t);
                                        's_3146: {
                                            if ast_stack.size == 0 {
                                                cur_node =
                                                    viml_pexpr_new_node(kExprNodeListLiteral);
                                                (*cur_node).start = cur_token.start;
                                                (*cur_node).len = cur_token.len;
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    == kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).start = prev_token.start;
                                                    (*cur_node).len = (*cur_node)
                                                        .len
                                                        .wrapping_add(prev_token.len);
                                                }
                                                (*cur_node).len = 0 as size_t;
                                                if want_node as ::core::ffi::c_uint
                                                    != kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).children = *top_node_p;
                                                }
                                                *top_node_p = cur_node;
                                                new_top_node_p = top_node_p;
                                            } else {
                                                if want_node as ::core::ffi::c_uint
                                                    == kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    if (***ast_stack.items.offset(
                                                        ast_stack
                                                            .size
                                                            .wrapping_sub(0 as size_t)
                                                            .wrapping_sub(1 as size_t)
                                                            as isize,
                                                    ))
                                                    .type_0
                                                        as ::core::ffi::c_uint
                                                        != kExprNodeListLiteral
                                                            as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                        && (***ast_stack.items.offset(
                                                            ast_stack
                                                                .size
                                                                .wrapping_sub(0 as size_t)
                                                                .wrapping_sub(1 as size_t)
                                                                as isize,
                                                        ))
                                                        .type_0
                                                            as ::core::ffi::c_uint
                                                            != kExprNodeComma as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                        && (***ast_stack.items.offset(
                                                            ast_stack
                                                                .size
                                                                .wrapping_sub(0 as size_t)
                                                                .wrapping_sub(1 as size_t)
                                                                as isize,
                                                        ))
                                                        .type_0
                                                            as ::core::ffi::c_uint
                                                            != kExprNodeColon as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                    {
                                                        is_invalid = true_0 != 0;
                                                        east_set_error(
                                                            pstate,
                                                            &raw mut ast.err,
                                                            gettext(
                                                                b"E15: Expected value, got closing bracket: %.*s\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                            ),
                                                            cur_token.start,
                                                        );
                                                    }
                                                }
                                                loop {
                                                    ast_stack.size = ast_stack.size.wrapping_sub(1);
                                                    new_top_node_p = *ast_stack
                                                        .items
                                                        .offset(ast_stack.size as isize);
                                                    if !(ast_stack.size != 0
                                                        && (new_top_node_p.is_null()
                                                            || (**new_top_node_p).type_0
                                                                as ::core::ffi::c_uint
                                                                != kExprNodeListLiteral
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                && (**new_top_node_p).type_0
                                                                    as ::core::ffi::c_uint
                                                                    != kExprNodeSubscript
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint))
                                                    {
                                                        break;
                                                    }
                                                }
                                                new_top_node = *new_top_node_p;
                                                match (*new_top_node).type_0 as ::core::ffi::c_uint
                                                {
                                                    6 => {
                                                        if pt_is_assignment(cur_pt)
                                                            as ::core::ffi::c_int
                                                            != 0
                                                            && (*new_top_node).children.is_null()
                                                        {
                                                            is_invalid = true_0 != 0;
                                                            east_set_error(
                                                                pstate,
                                                                &raw mut ast.err,
                                                                gettext(
                                                                    b"E475: Unable to assign to empty list: %.*s\0".as_ptr()
                                                                        as *const ::core::ffi::c_char,
                                                                ),
                                                                cur_token.start,
                                                            );
                                                        }
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidList\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimList\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_3146;
                                                    }
                                                    5 => {
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidSubscriptBracket\0"
                                                                    .as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimSubscriptBracket\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_3146;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            '_c2rust_label_21: {
                                                if ast_stack.size == 0 {
                                                } else {
                                                    __assert_fail(
                                                        b"!kv_size(ast_stack)\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2458 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Unexpected closing figure brace: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidList\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimList\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        if ast_stack.size == ast_stack.capacity {
                                            ast_stack.capacity =
                                                if ast_stack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                                };
                                            ast_stack.items =
                                                (if ast_stack.capacity
                                                    == ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            ast_stack.items
                                                                as *const ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut *mut *mut ExprASTNode;
                                        } else {
                                        };
                                        let c2rust_fresh11 = ast_stack.size;
                                        ast_stack.size = ast_stack.size.wrapping_add(1);
                                        let c2rust_lvalue_ptr_4 = &raw mut *ast_stack
                                            .items
                                            .offset(c2rust_fresh11 as isize);
                                        *c2rust_lvalue_ptr_4 = new_top_node_p;
                                        want_node = kENodeOperator;
                                        if ast_stack.size <= asgn_level {
                                            '_c2rust_label_22: {
                                                if ast_stack.size == asgn_level {
                                                } else {
                                                    __assert_fail(
                                                        b"kv_size(ast_stack) == asgn_level\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2466 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            asgn_level = 0 as size_t;
                                            if cur_pt as ::core::ffi::c_uint
                                                == kEPTAssignment as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                '_c2rust_label_23: {
                                                    if !ast.err.msg.is_null() {
                                                    } else {
                                                        __assert_fail(
                                                            b"ast.err.msg\0".as_ptr() as *const ::core::ffi::c_char,
                                                            b"src/nvim/viml/parser/expressions.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            2469 as ::core::ffi::c_uint,
                                                            b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                    }
                                                };
                                            } else if cur_pt as ::core::ffi::c_uint
                                                == kEPTExpr as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                && pt_stack.size > 1 as size_t
                                                && pt_is_assignment(
                                                    *pt_stack.items.offset(
                                                        pt_stack
                                                            .size
                                                            .wrapping_sub(1 as size_t)
                                                            .wrapping_sub(1 as size_t)
                                                            as isize,
                                                    ),
                                                )
                                                    as ::core::ffi::c_int
                                                    != 0
                                            {
                                                pt_stack.size =
                                                    pt_stack.size.wrapping_sub(1 as size_t);
                                            }
                                        }
                                        if cur_pt as ::core::ffi::c_uint
                                            == kEPTSingleAssignment as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            && ast_stack.size == 1 as size_t
                                        {
                                            pt_stack.size = pt_stack.size.wrapping_sub(1 as size_t);
                                        }
                                        break '_viml_pexpr_parse_cycle_end;
                                    } else if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        cur_node = viml_pexpr_new_node(kExprNodeListLiteral);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        *top_node_p = cur_node;
                                        if ast_stack.size == ast_stack.capacity {
                                            ast_stack.capacity =
                                                if ast_stack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                                };
                                            ast_stack.items =
                                                (if ast_stack.capacity
                                                    == ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            ast_stack.items
                                                                as *const ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut *mut *mut ExprASTNode;
                                        } else {
                                        };
                                        let c2rust_fresh12 = ast_stack.size;
                                        ast_stack.size = ast_stack.size.wrapping_add(1);
                                        let c2rust_lvalue_ptr_5 = &raw mut *ast_stack
                                            .items
                                            .offset(c2rust_fresh12 as isize);
                                        *c2rust_lvalue_ptr_5 = &raw mut (*cur_node).children;
                                        if cur_pt as ::core::ffi::c_uint
                                            == kEPTAssignment as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            if pt_stack.size == pt_stack.capacity {
                                                pt_stack.capacity = if pt_stack.capacity
                                                    << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[ExprASTParseType; 4]>(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        ExprASTParseType,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [ExprASTParseType; 4],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                    pt_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[ExprASTParseType; 4]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [ExprASTParseType; 4],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                ExprASTParseType,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                                pt_stack.items =
                                                    (if pt_stack.capacity
                                                        == ::core::mem::size_of::<
                                                            [ExprASTParseType; 4],
                                                        >(
                                                        )
                                                        .wrapping_div(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [ExprASTParseType; 4],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                ExprASTParseType,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        )
                                                    {
                                                        if pt_stack.items
                                                            == &raw mut pt_stack.init_array
                                                                as *mut ExprASTParseType
                                                        {
                                                            pt_stack.items
                                                                as *mut ::core::ffi::c_void
                                                        } else {
                                                            _memcpy_free(
                                                                &raw mut pt_stack.init_array
                                                                    as *mut ExprASTParseType
                                                                    as *mut ::core::ffi::c_void,
                                                                pt_stack.items
                                                                    as *mut ::core::ffi::c_void,
                                                                pt_stack.size.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        }
                                                    } else {
                                                        if pt_stack.items
                                                            == &raw mut pt_stack.init_array
                                                                as *mut ExprASTParseType
                                                        {
                                                            memcpy(
                                                                xmalloc(
                                                                    pt_stack.capacity.wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            ExprASTParseType,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                ),
                                                                pt_stack.items
                                                                    as *const ::core::ffi::c_void,
                                                                pt_stack.size.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        } else {
                                                            xrealloc(
                                                                pt_stack.items
                                                                    as *mut ::core::ffi::c_void,
                                                                pt_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        }
                                                    })
                                                        as *mut ExprASTParseType;
                                            } else {
                                            };
                                            let c2rust_fresh13 = pt_stack.size;
                                            pt_stack.size = pt_stack.size.wrapping_add(1);
                                            *pt_stack.items.offset(c2rust_fresh13 as isize) =
                                                kEPTSingleAssignment;
                                        } else if cur_pt as ::core::ffi::c_uint
                                            == kEPTSingleAssignment as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E475: Nested lists not allowed when assigning: %.*s\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                        }
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidList\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimList\0".as_ptr() as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    } else if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.size == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        '_c2rust_label_24: {
                                            if !(*top_node_p).is_null() {
                                            } else {
                                                __assert_fail(
                                                    b"*top_node_p != NULL\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2499 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeSubscript);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidSubscriptBracket\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimSubscriptBracket\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        if pt_is_assignment(cur_pt) {
                                            '_c2rust_label_25: {
                                                if want_node as ::core::ffi::c_uint
                                                    == kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"want_node == kENodeValue\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2505 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            asgn_level = ast_stack.size.wrapping_sub(1 as size_t);
                                            if pt_stack.size == pt_stack.capacity {
                                                pt_stack.capacity = if pt_stack.capacity
                                                    << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[ExprASTParseType; 4]>(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        ExprASTParseType,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [ExprASTParseType; 4],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                    pt_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[ExprASTParseType; 4]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [ExprASTParseType; 4],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                ExprASTParseType,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                                pt_stack.items =
                                                    (if pt_stack.capacity
                                                        == ::core::mem::size_of::<
                                                            [ExprASTParseType; 4],
                                                        >(
                                                        )
                                                        .wrapping_div(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [ExprASTParseType; 4],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                ExprASTParseType,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        )
                                                    {
                                                        if pt_stack.items
                                                            == &raw mut pt_stack.init_array
                                                                as *mut ExprASTParseType
                                                        {
                                                            pt_stack.items
                                                                as *mut ::core::ffi::c_void
                                                        } else {
                                                            _memcpy_free(
                                                                &raw mut pt_stack.init_array
                                                                    as *mut ExprASTParseType
                                                                    as *mut ::core::ffi::c_void,
                                                                pt_stack.items
                                                                    as *mut ::core::ffi::c_void,
                                                                pt_stack.size.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        }
                                                    } else {
                                                        if pt_stack.items
                                                            == &raw mut pt_stack.init_array
                                                                as *mut ExprASTParseType
                                                        {
                                                            memcpy(
                                                                xmalloc(
                                                                    pt_stack.capacity.wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            ExprASTParseType,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                ),
                                                                pt_stack.items
                                                                    as *const ::core::ffi::c_void,
                                                                pt_stack.size.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        } else {
                                                            xrealloc(
                                                                pt_stack.items
                                                                    as *mut ::core::ffi::c_void,
                                                                pt_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        }
                                                    })
                                                        as *mut ExprASTParseType;
                                            } else {
                                            };
                                            let c2rust_fresh14 = pt_stack.size;
                                            pt_stack.size = pt_stack.size.wrapping_add(1);
                                            *pt_stack.items.offset(c2rust_fresh14 as isize) =
                                                kEPTExpr;
                                        }
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                22 => {
                                    if cur_token.data.brc.closing {
                                        let mut new_top_node_0: *mut ExprASTNode =
                                            ::core::ptr::null_mut::<ExprASTNode>();
                                        let mut new_top_node_p_0: *mut *mut ExprASTNode =
                                            ::core::ptr::null_mut::<*mut ExprASTNode>();
                                        ast_stack.size = ast_stack.size.wrapping_sub(1 as size_t);
                                        's_3806: {
                                            if ast_stack.size == 0 {
                                                cur_node =
                                                    viml_pexpr_new_node(kExprNodeUnknownFigure);
                                                (*cur_node).start = cur_token.start;
                                                (*cur_node).len = cur_token.len;
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    == kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).start = prev_token.start;
                                                    (*cur_node).len = (*cur_node)
                                                        .len
                                                        .wrapping_add(prev_token.len);
                                                }
                                                (*cur_node).data.fig.type_guesses.allow_lambda =
                                                    false_0 != 0;
                                                (*cur_node).data.fig.type_guesses.allow_dict =
                                                    false_0 != 0;
                                                (*cur_node).data.fig.type_guesses.allow_ident =
                                                    false_0 != 0;
                                                (*cur_node).len = 0 as size_t;
                                                if want_node as ::core::ffi::c_uint
                                                    != kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).children = *top_node_p;
                                                }
                                                *top_node_p = cur_node;
                                                new_top_node_p_0 = top_node_p;
                                            } else {
                                                if want_node as ::core::ffi::c_uint
                                                    == kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    if (***ast_stack.items.offset(
                                                        ast_stack
                                                            .size
                                                            .wrapping_sub(0 as size_t)
                                                            .wrapping_sub(1 as size_t)
                                                            as isize,
                                                    ))
                                                    .type_0
                                                        as ::core::ffi::c_uint
                                                        != kExprNodeUnknownFigure
                                                            as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                        && (***ast_stack.items.offset(
                                                            ast_stack
                                                                .size
                                                                .wrapping_sub(0 as size_t)
                                                                .wrapping_sub(1 as size_t)
                                                                as isize,
                                                        ))
                                                        .type_0
                                                            as ::core::ffi::c_uint
                                                            != kExprNodeComma as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                    {
                                                        is_invalid = true_0 != 0;
                                                        east_set_error(
                                                            pstate,
                                                            &raw mut ast.err,
                                                            gettext(
                                                                b"E15: Expected value, got closing figure brace: %.*s\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                            ),
                                                            cur_token.start,
                                                        );
                                                    }
                                                }
                                                loop {
                                                    ast_stack.size = ast_stack.size.wrapping_sub(1);
                                                    new_top_node_p_0 = *ast_stack
                                                        .items
                                                        .offset(ast_stack.size as isize);
                                                    if !(ast_stack.size != 0
                                                        && (new_top_node_p_0.is_null()
                                                            || (**new_top_node_p_0).type_0 as ::core::ffi::c_uint
                                                                != kExprNodeUnknownFigure as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                && (**new_top_node_p_0).type_0 as ::core::ffi::c_uint
                                                                    != kExprNodeDictLiteral as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                && (**new_top_node_p_0).type_0 as ::core::ffi::c_uint
                                                                    != kExprNodeCurlyBracesIdentifier as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                && (**new_top_node_p_0).type_0 as ::core::ffi::c_uint
                                                                    != kExprNodeLambda as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint))
                                                    {
                                                        break;
                                                    }
                                                }
                                                new_top_node_0 = *new_top_node_p_0;
                                                match (*new_top_node_0).type_0
                                                    as ::core::ffi::c_uint
                                                {
                                                    14 => {
                                                        if (*new_top_node_0).children.is_null() {
                                                            '_c2rust_label_26: {
                                                                if want_node as ::core::ffi::c_uint
                                                                    == kENodeValue
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                {
                                                                } else {
                                                                    __assert_fail(
                                                                        b"want_node == kENodeValue\0".as_ptr()
                                                                            as *const ::core::ffi::c_char,
                                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                        2558 as ::core::ffi::c_uint,
                                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                    );
                                                                }
                                                            };
                                                            '_c2rust_label_27: {
                                                                if (*new_top_node_0)
                                                                    .data
                                                                    .fig
                                                                    .type_guesses
                                                                    .allow_dict
                                                                {
                                                                } else {
                                                                    __assert_fail(
                                                                        b"new_top_node->data.fig.type_guesses.allow_dict\0".as_ptr()
                                                                            as *const ::core::ffi::c_char,
                                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                        2559 as ::core::ffi::c_uint,
                                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                    );
                                                                }
                                                            };
                                                            let node__1: *mut ExprASTNode =
                                                                new_top_node_0;
                                                            '_c2rust_label_28: {
                                                                if (*node__1).type_0
                                                                    as ::core::ffi::c_uint
                                                                    == kExprNodeUnknownFigure
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                    || (*node__1).type_0
                                                                        as ::core::ffi::c_uint
                                                                        == kExprNodeDictLiteral
                                                                            as ::core::ffi::c_int
                                                                            as ::core::ffi::c_uint
                                                                {
                                                                } else {
                                                                    __assert_fail(
                                                                        b"node_->type == kExprNodeUnknownFigure || node_->type == kExprNodeDictLiteral\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                        2560 as ::core::ffi::c_uint,
                                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                    );
                                                                }
                                                            };
                                                            (*node__1).type_0 =
                                                                kExprNodeDictLiteral;
                                                            if !(*pstate).colors.is_null() {
                                                                (*(*(*pstate).colors)
                                                                    .items
                                                                    .offset(
                                                                        (*node__1)
                                                                            .data
                                                                            .fig
                                                                            .opening_hl_idx
                                                                            as isize,
                                                                    ))
                                                                .group = if is_invalid
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                };
                                                            }
                                                            viml_parser_highlight(
                                                                pstate,
                                                                cur_token.start,
                                                                cur_token.len,
                                                                if is_invalid as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                },
                                                            );
                                                        } else if (*new_top_node_0)
                                                            .data
                                                            .fig
                                                            .type_guesses
                                                            .allow_ident
                                                        {
                                                            let node__2: *mut ExprASTNode =
                                                                new_top_node_0;
                                                            '_c2rust_label_29: {
                                                                if (*node__2).type_0 as ::core::ffi::c_uint
                                                                    == kExprNodeUnknownFigure as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                    || (*node__2).type_0 as ::core::ffi::c_uint
                                                                        == kExprNodeCurlyBracesIdentifier as ::core::ffi::c_int
                                                                            as ::core::ffi::c_uint
                                                                {} else {
                                                                    __assert_fail(
                                                                        b"node_->type == kExprNodeUnknownFigure || node_->type == kExprNodeCurlyBracesIdentifier\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                        2564 as ::core::ffi::c_uint,
                                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                                    );
                                                                }
                                                            };
                                                            (*node__2).type_0 =
                                                                kExprNodeCurlyBracesIdentifier;
                                                            if !(*pstate).colors.is_null() {
                                                                (*(*(*pstate).colors)
                                                                    .items
                                                                    .offset(
                                                                        (*node__2)
                                                                            .data
                                                                            .fig
                                                                            .opening_hl_idx
                                                                            as isize,
                                                                    ))
                                                                .group = if is_invalid
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidCurly\0".as_ptr() as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimCurly\0".as_ptr() as *const ::core::ffi::c_char
                                                                };
                                                            }
                                                            viml_parser_highlight(
                                                                pstate,
                                                                cur_token.start,
                                                                cur_token.len,
                                                                if is_invalid as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidCurly\0".as_ptr() as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimCurly\0".as_ptr() as *const ::core::ffi::c_char
                                                                },
                                                            );
                                                        } else {
                                                            is_invalid = true_0 != 0;
                                                            east_set_error(
                                                                pstate,
                                                                &raw mut ast.err,
                                                                gettext(
                                                                    b"E15: Don't know what figure brace means: %.*s\0".as_ptr()
                                                                        as *const ::core::ffi::c_char,
                                                                ),
                                                                (*new_top_node_0).start,
                                                            );
                                                            if !(*pstate).colors.is_null() {
                                                                (*(*(*pstate).colors)
                                                                    .items
                                                                    .offset(
                                                                        (*new_top_node_0)
                                                                            .data
                                                                            .fig
                                                                            .opening_hl_idx
                                                                            as isize,
                                                                    ))
                                                                .group = if is_invalid
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidFigureBrace\0".as_ptr()
                                                                        as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimFigureBrace\0".as_ptr() as *const ::core::ffi::c_char
                                                                };
                                                            }
                                                            viml_parser_highlight(
                                                                pstate,
                                                                cur_token.start,
                                                                cur_token.len,
                                                                if is_invalid as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidFigureBrace\0".as_ptr()
                                                                        as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimFigureBrace\0".as_ptr() as *const ::core::ffi::c_char
                                                                },
                                                            );
                                                        }
                                                        break 's_3806;
                                                    }
                                                    16 => {
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidDict\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimDict\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_3806;
                                                    }
                                                    17 => {
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidCurly\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimCurly\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_3806;
                                                    }
                                                    15 => {
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidLambda\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimLambda\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_3806;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            '_c2rust_label_30: {
                                                if ast_stack.size == 0 {
                                                } else {
                                                    __assert_fail(
                                                        b"!kv_size(ast_stack)\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2592 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Unexpected closing figure brace: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidFigureBrace\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimFigureBrace\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        if ast_stack.size == ast_stack.capacity {
                                            ast_stack.capacity =
                                                if ast_stack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                                };
                                            ast_stack.items =
                                                (if ast_stack.capacity
                                                    == ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            ast_stack.items
                                                                as *const ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut *mut *mut ExprASTNode;
                                        } else {
                                        };
                                        let c2rust_fresh15 = ast_stack.size;
                                        ast_stack.size = ast_stack.size.wrapping_add(1);
                                        let c2rust_lvalue_ptr_6 = &raw mut *ast_stack
                                            .items
                                            .offset(c2rust_fresh15 as isize);
                                        *c2rust_lvalue_ptr_6 = new_top_node_p_0;
                                        want_node = kENodeOperator;
                                        if ast_stack.size <= asgn_level {
                                            '_c2rust_label_31: {
                                                if ast_stack.size == asgn_level {
                                                } else {
                                                    __assert_fail(
                                                        b"kv_size(ast_stack) == asgn_level\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2600 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            if cur_pt as ::core::ffi::c_uint
                                                == kEPTExpr as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                && pt_stack.size > 1 as size_t
                                                && pt_is_assignment(
                                                    *pt_stack.items.offset(
                                                        pt_stack
                                                            .size
                                                            .wrapping_sub(1 as size_t)
                                                            .wrapping_sub(1 as size_t)
                                                            as isize,
                                                    ),
                                                )
                                                    as ::core::ffi::c_int
                                                    != 0
                                            {
                                                pt_stack.size =
                                                    pt_stack.size.wrapping_sub(1 as size_t);
                                                asgn_level = 0 as size_t;
                                            }
                                        }
                                        break '_viml_pexpr_parse_cycle_end;
                                    } else if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidFigureBrace\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimFigureBrace\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        if pt_is_assignment(cur_pt) {
                                            cur_node =
                                                viml_pexpr_new_node(kExprNodeCurlyBracesIdentifier);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).data.fig.type_guesses.allow_lambda =
                                                false_0 != 0;
                                            (*cur_node).data.fig.type_guesses.allow_dict =
                                                false_0 != 0;
                                            (*cur_node).data.fig.type_guesses.allow_ident =
                                                true_0 != 0;
                                            if pt_stack.size == pt_stack.capacity {
                                                pt_stack.capacity = if pt_stack.capacity
                                                    << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[ExprASTParseType; 4]>(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        ExprASTParseType,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [ExprASTParseType; 4],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                    pt_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[ExprASTParseType; 4]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [ExprASTParseType; 4],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                ExprASTParseType,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                                pt_stack.items =
                                                    (if pt_stack.capacity
                                                        == ::core::mem::size_of::<
                                                            [ExprASTParseType; 4],
                                                        >(
                                                        )
                                                        .wrapping_div(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [ExprASTParseType; 4],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                ExprASTParseType,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        )
                                                    {
                                                        if pt_stack.items
                                                            == &raw mut pt_stack.init_array
                                                                as *mut ExprASTParseType
                                                        {
                                                            pt_stack.items
                                                                as *mut ::core::ffi::c_void
                                                        } else {
                                                            _memcpy_free(
                                                                &raw mut pt_stack.init_array
                                                                    as *mut ExprASTParseType
                                                                    as *mut ::core::ffi::c_void,
                                                                pt_stack.items
                                                                    as *mut ::core::ffi::c_void,
                                                                pt_stack.size.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        }
                                                    } else {
                                                        if pt_stack.items
                                                            == &raw mut pt_stack.init_array
                                                                as *mut ExprASTParseType
                                                        {
                                                            memcpy(
                                                                xmalloc(
                                                                    pt_stack.capacity.wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            ExprASTParseType,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                ),
                                                                pt_stack.items
                                                                    as *const ::core::ffi::c_void,
                                                                pt_stack.size.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        } else {
                                                            xrealloc(
                                                                pt_stack.items
                                                                    as *mut ::core::ffi::c_void,
                                                                pt_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        }
                                                    })
                                                        as *mut ExprASTParseType;
                                            } else {
                                            };
                                            let c2rust_fresh16 = pt_stack.size;
                                            pt_stack.size = pt_stack.size.wrapping_add(1);
                                            *pt_stack.items.offset(c2rust_fresh16 as isize) =
                                                kEPTExpr;
                                        } else {
                                            cur_node = viml_pexpr_new_node(kExprNodeUnknownFigure);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).data.fig.type_guesses.allow_lambda =
                                                true_0 != 0;
                                            (*cur_node).data.fig.type_guesses.allow_dict =
                                                true_0 != 0;
                                            (*cur_node).data.fig.type_guesses.allow_ident =
                                                true_0 != 0;
                                        }
                                        if !(*pstate).colors.is_null() {
                                            (*cur_node).data.fig.opening_hl_idx =
                                                (*(*pstate).colors).size.wrapping_sub(1 as size_t);
                                        }
                                        *top_node_p = cur_node;
                                        if ast_stack.size == ast_stack.capacity {
                                            ast_stack.capacity =
                                                if ast_stack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                                };
                                            ast_stack.items =
                                                (if ast_stack.capacity
                                                    == ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            ast_stack.items
                                                                as *const ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut *mut *mut ExprASTNode;
                                        } else {
                                        };
                                        let c2rust_fresh17 = ast_stack.size;
                                        ast_stack.size = ast_stack.size.wrapping_add(1);
                                        let c2rust_lvalue_ptr_7 = &raw mut *ast_stack
                                            .items
                                            .offset(c2rust_fresh17 as isize);
                                        *c2rust_lvalue_ptr_7 = &raw mut (*cur_node).children;
                                        if pt_stack.size == pt_stack.capacity {
                                            pt_stack.capacity = if pt_stack.capacity
                                                << 1 as ::core::ffi::c_int
                                                > ::core::mem::size_of::<[ExprASTParseType; 4]>()
                                                    .wrapping_div(::core::mem::size_of::<
                                                        ExprASTParseType,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [ExprASTParseType; 4],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                pt_stack.capacity << 1 as ::core::ffi::c_int
                                            } else {
                                                ::core::mem::size_of::<[ExprASTParseType; 4]>()
                                                    .wrapping_div(::core::mem::size_of::<
                                                        ExprASTParseType,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [ExprASTParseType; 4],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                            };
                                            pt_stack.items = (if pt_stack.capacity
                                                == ::core::mem::size_of::<[ExprASTParseType; 4]>()
                                                    .wrapping_div(::core::mem::size_of::<
                                                        ExprASTParseType,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [ExprASTParseType; 4],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            ExprASTParseType,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                if pt_stack.items
                                                    == &raw mut pt_stack.init_array
                                                        as *mut ExprASTParseType
                                                {
                                                    pt_stack.items as *mut ::core::ffi::c_void
                                                } else {
                                                    _memcpy_free(
                                                        &raw mut pt_stack.init_array as *mut ExprASTParseType
                                                            as *mut ::core::ffi::c_void,
                                                        pt_stack.items as *mut ::core::ffi::c_void,
                                                        pt_stack
                                                            .size
                                                            .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                                                    )
                                                }
                                            } else {
                                                if pt_stack.items
                                                    == &raw mut pt_stack.init_array
                                                        as *mut ExprASTParseType
                                                {
                                                    memcpy(
                                                        xmalloc(
                                                            pt_stack
                                                                .capacity
                                                                .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                                                        ),
                                                        pt_stack.items as *const ::core::ffi::c_void,
                                                        pt_stack
                                                            .size
                                                            .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                                                    )
                                                } else {
                                                    xrealloc(
                                                        pt_stack.items as *mut ::core::ffi::c_void,
                                                        pt_stack
                                                            .capacity
                                                            .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                                                    )
                                                }
                                            })
                                                as *mut ExprASTParseType;
                                        } else {
                                        };
                                        let c2rust_fresh18 = pt_stack.size;
                                        pt_stack.size = pt_stack.size.wrapping_add(1);
                                        *pt_stack.items.offset(c2rust_fresh18 as isize) =
                                            kEPTLambdaArguments;
                                        lambda_node = cur_node;
                                        break 's_4376;
                                    } else {
                                        '_c2rust_label_32: {
                                            if want_node as ::core::ffi::c_uint
                                                == kENodeOperator as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"want_node == kENodeOperator\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2652 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                                && ast_stack.size == 1 as size_t
                                            {
                                                break '_viml_pexpr_parse_end;
                                            }
                                            '_c2rust_label_33: {
                                                if !(*top_node_p).is_null() {
                                                } else {
                                                    __assert_fail(
                                                        b"*top_node_p != NULL\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2652 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                cur_token.start,
                                            );
                                            cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).len = 0 as size_t;
                                            is_invalid = is_invalid as ::core::ffi::c_int
                                                | !viml_pexpr_handle_bop(
                                                    pstate,
                                                    &raw mut ast_stack,
                                                    cur_node,
                                                    &raw mut want_node,
                                                    &raw mut ast.err,
                                                )
                                                    as ::core::ffi::c_int
                                                != 0;
                                        } else {
                                            match (**top_node_p).type_0 as ::core::ffi::c_uint {
                                                13 | 11 | 17 => {
                                                    cur_node = viml_pexpr_new_node(
                                                        kExprNodeComplexIdentifier,
                                                    );
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    (*cur_node).len = 0 as size_t;
                                                    (*cur_node).children = *top_node_p;
                                                    *top_node_p = cur_node;
                                                    if ast_stack.size == ast_stack.capacity {
                                                        ast_stack.capacity = if ast_stack.capacity
                                                            << 1 as ::core::ffi::c_int
                                                            > ::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [*mut *mut ExprASTNode; 16],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            ) {
                                                            ast_stack.capacity
                                                                << 1 as ::core::ffi::c_int
                                                        } else {
                                                            ::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [*mut *mut ExprASTNode; 16],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as size_t,
                                                            )
                                                        };
                                                        ast_stack.items = (if ast_stack.capacity
                                                            == ::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [*mut *mut ExprASTNode; 16],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            ) {
                                                            if ast_stack.items
                                                                == &raw mut ast_stack.init_array
                                                                    as *mut *mut *mut ExprASTNode
                                                            {
                                                                ast_stack.items
                                                                    as *mut ::core::ffi::c_void
                                                            } else {
                                                                _memcpy_free(
                                                                    &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                                                                        as *mut ::core::ffi::c_void,
                                                                    ast_stack.items as *mut ::core::ffi::c_void,
                                                                    ast_stack
                                                                        .size
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                        ),
                                                                )
                                                            }
                                                        } else {
                                                            if ast_stack.items
                                                                == &raw mut ast_stack.init_array
                                                                    as *mut *mut *mut ExprASTNode
                                                            {
                                                                memcpy(
                                                                    xmalloc(
                                                                        ast_stack
                                                                            .capacity
                                                                            .wrapping_mul(
                                                                                ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                            ),
                                                                    ),
                                                                    ast_stack.items as *const ::core::ffi::c_void,
                                                                    ast_stack
                                                                        .size
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                        ),
                                                                )
                                                            } else {
                                                                xrealloc(
                                                                    ast_stack.items
                                                                        as *mut ::core::ffi::c_void,
                                                                    ast_stack
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            *mut *mut ExprASTNode,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                )
                                                            }
                                                        })
                                                            as *mut *mut *mut ExprASTNode;
                                                    } else {
                                                    };
                                                    let c2rust_fresh19 = ast_stack.size;
                                                    ast_stack.size = ast_stack.size.wrapping_add(1);
                                                    let c2rust_lvalue_ptr_8 = &raw mut *ast_stack
                                                        .items
                                                        .offset(c2rust_fresh19 as isize);
                                                    *c2rust_lvalue_ptr_8 =
                                                        &raw mut (*(*cur_node).children).next;
                                                    let new_top_node_p_1: *mut *mut ExprASTNode =
                                                        *ast_stack.items.offset(
                                                            ast_stack
                                                                .size
                                                                .wrapping_sub(0 as size_t)
                                                                .wrapping_sub(1 as size_t)
                                                                as isize,
                                                        );
                                                    '_c2rust_label_34: {
                                                        if (*new_top_node_p_1).is_null() {
                                                        } else {
                                                            __assert_fail(
                                                                b"*new_top_node_p == NULL\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                                b"src/nvim/viml/parser/expressions.rs\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                                2652 as ::core::ffi::c_uint,
                                                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                            );
                                                        }
                                                    };
                                                    cur_node = viml_pexpr_new_node(
                                                        kExprNodeCurlyBracesIdentifier,
                                                    );
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    if !(*pstate).colors.is_null() {
                                                        (*cur_node).data.fig.opening_hl_idx =
                                                            (*(*pstate).colors).size;
                                                    }
                                                    (*cur_node)
                                                        .data
                                                        .fig
                                                        .type_guesses
                                                        .allow_lambda = false;
                                                    (*cur_node).data.fig.type_guesses.allow_dict =
                                                        false;
                                                    (*cur_node).data.fig.type_guesses.allow_ident =
                                                        true;
                                                    if ast_stack.size == ast_stack.capacity {
                                                        ast_stack.capacity = if ast_stack.capacity
                                                            << 1 as ::core::ffi::c_int
                                                            > ::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [*mut *mut ExprASTNode; 16],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            ) {
                                                            ast_stack.capacity
                                                                << 1 as ::core::ffi::c_int
                                                        } else {
                                                            ::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [*mut *mut ExprASTNode; 16],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as size_t,
                                                            )
                                                        };
                                                        ast_stack.items = (if ast_stack.capacity
                                                            == ::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [*mut *mut ExprASTNode; 16],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            ) {
                                                            if ast_stack.items
                                                                == &raw mut ast_stack.init_array
                                                                    as *mut *mut *mut ExprASTNode
                                                            {
                                                                ast_stack.items
                                                                    as *mut ::core::ffi::c_void
                                                            } else {
                                                                _memcpy_free(
                                                                    &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                                                                        as *mut ::core::ffi::c_void,
                                                                    ast_stack.items as *mut ::core::ffi::c_void,
                                                                    ast_stack
                                                                        .size
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                        ),
                                                                )
                                                            }
                                                        } else {
                                                            if ast_stack.items
                                                                == &raw mut ast_stack.init_array
                                                                    as *mut *mut *mut ExprASTNode
                                                            {
                                                                memcpy(
                                                                    xmalloc(
                                                                        ast_stack
                                                                            .capacity
                                                                            .wrapping_mul(
                                                                                ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                            ),
                                                                    ),
                                                                    ast_stack.items as *const ::core::ffi::c_void,
                                                                    ast_stack
                                                                        .size
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                        ),
                                                                )
                                                            } else {
                                                                xrealloc(
                                                                    ast_stack.items
                                                                        as *mut ::core::ffi::c_void,
                                                                    ast_stack
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            *mut *mut ExprASTNode,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                )
                                                            }
                                                        })
                                                            as *mut *mut *mut ExprASTNode;
                                                    } else {
                                                    };
                                                    let c2rust_fresh20 = ast_stack.size;
                                                    ast_stack.size = ast_stack.size.wrapping_add(1);
                                                    let c2rust_lvalue_ptr_9 = &raw mut *ast_stack
                                                        .items
                                                        .offset(c2rust_fresh20 as isize);
                                                    *c2rust_lvalue_ptr_9 =
                                                        &raw mut (*cur_node).children;
                                                    if pt_is_assignment(cur_pt) {
                                                        if pt_stack.size == pt_stack.capacity {
                                                            pt_stack.capacity = if pt_stack.capacity
                                                                << 1 as ::core::ffi::c_int
                                                                > ::core::mem::size_of::<
                                                                    [ExprASTParseType; 4],
                                                                >(
                                                                )
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [ExprASTParseType; 4],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            ExprASTParseType,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as usize,
                                                                ) {
                                                                pt_stack.capacity
                                                                    << 1 as ::core::ffi::c_int
                                                            } else {
                                                                ::core::mem::size_of::<
                                                                    [ExprASTParseType; 4],
                                                                >(
                                                                )
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [ExprASTParseType; 4],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            ExprASTParseType,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as size_t,
                                                                )
                                                            };
                                                            pt_stack.items = (if pt_stack.capacity
                                                                == ::core::mem::size_of::<
                                                                    [ExprASTParseType; 4],
                                                                >(
                                                                )
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<
                                                                        ExprASTParseType,
                                                                    >(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [ExprASTParseType; 4],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            ExprASTParseType,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as usize,
                                                                ) {
                                                                if pt_stack.items
                                                                    == &raw mut pt_stack.init_array
                                                                        as *mut ExprASTParseType
                                                                {
                                                                    pt_stack.items
                                                                        as *mut ::core::ffi::c_void
                                                                } else {
                                                                    _memcpy_free(
                                                                        &raw mut pt_stack.init_array as *mut ExprASTParseType
                                                                            as *mut ::core::ffi::c_void,
                                                                        pt_stack.items as *mut ::core::ffi::c_void,
                                                                        pt_stack
                                                                            .size
                                                                            .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                                                                    )
                                                                }
                                                            } else {
                                                                if pt_stack.items
                                                                    == &raw mut pt_stack.init_array
                                                                        as *mut ExprASTParseType
                                                                {
                                                                    memcpy(
                                                                        xmalloc(
                                                                            pt_stack
                                                                                .capacity
                                                                                .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                                                                        ),
                                                                        pt_stack.items as *const ::core::ffi::c_void,
                                                                        pt_stack
                                                                            .size
                                                                            .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                                                                    )
                                                                } else {
                                                                    xrealloc(
                                                                        pt_stack.items as *mut ::core::ffi::c_void,
                                                                        pt_stack
                                                                            .capacity
                                                                            .wrapping_mul(::core::mem::size_of::<ExprASTParseType>()),
                                                                    )
                                                                }
                                                            })
                                                                as *mut ExprASTParseType;
                                                        } else {
                                                        };
                                                        let c2rust_fresh21 = pt_stack.size;
                                                        pt_stack.size =
                                                            pt_stack.size.wrapping_add(1);
                                                        *pt_stack
                                                            .items
                                                            .offset(c2rust_fresh21 as isize) =
                                                            kEPTExpr;
                                                    }
                                                    want_node = kENodeValue;
                                                    *new_top_node_p_1 = cur_node;
                                                    viml_parser_highlight(
                                                        pstate,
                                                        cur_token.start,
                                                        cur_token.len,
                                                        if is_invalid as ::core::ffi::c_int != 0 {
                                                            b"NvimInvalidCurly\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                        } else {
                                                            b"NvimCurly\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                        },
                                                    );
                                                    break 's_4376;
                                                }
                                                _ => {
                                                    if flags & kExprFlagsMulti as ::core::ffi::c_int
                                                        != 0
                                                        && ast_stack.size == 1 as size_t
                                                    {
                                                        break '_viml_pexpr_parse_end;
                                                    }
                                                    '_c2rust_label_35: {
                                                        if !(*top_node_p).is_null() {
                                                        } else {
                                                            __assert_fail(
                                                                b"*top_node_p != NULL\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                                b"src/nvim/viml/parser/expressions.rs\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                                2652 as ::core::ffi::c_uint,
                                                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                            );
                                                        }
                                                    };
                                                    is_invalid = true_0 != 0;
                                                    east_set_error(
                                                        pstate,
                                                        &raw mut ast.err,
                                                        gettext(
                                                            b"E15: Missing operator: %.*s\0"
                                                                .as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                        cur_token.start,
                                                    );
                                                    cur_node =
                                                        viml_pexpr_new_node(kExprNodeOpMissing);
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    (*cur_node).len = 0 as size_t;
                                                    is_invalid = is_invalid as ::core::ffi::c_int
                                                        | !viml_pexpr_handle_bop(
                                                            pstate,
                                                            &raw mut ast_stack,
                                                            cur_node,
                                                            &raw mut want_node,
                                                            &raw mut ast.err,
                                                        )
                                                            as ::core::ffi::c_int
                                                        != 0;
                                                }
                                            }
                                        }
                                    }
                                }
                                25 => {
                                    if cur_pt as ::core::ffi::c_uint
                                        == kEPTLambdaArguments as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        pt_stack.size = pt_stack.size.wrapping_sub(1 as size_t);
                                        '_c2rust_label_37: {
                                            if pt_stack.size != 0 {
                                            } else {
                                                __assert_fail(
                                                    b"kv_size(pt_stack)\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2665 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        if want_node as ::core::ffi::c_uint
                                            == kENodeValue as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            ast_stack.size =
                                                ast_stack.size.wrapping_sub(1 as size_t);
                                        }
                                        '_c2rust_label_38: {
                                            if ast_stack.size >= 1 as size_t {
                                            } else {
                                                __assert_fail(
                                                    b"kv_size(ast_stack) >= 1\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2671 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        while (***ast_stack.items.offset(
                                            ast_stack
                                                .size
                                                .wrapping_sub(0 as size_t)
                                                .wrapping_sub(1 as size_t)
                                                as isize,
                                        ))
                                        .type_0
                                            as ::core::ffi::c_uint
                                            != kExprNodeLambda as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            && (***ast_stack.items.offset(
                                                ast_stack
                                                    .size
                                                    .wrapping_sub(0 as size_t)
                                                    .wrapping_sub(1 as size_t)
                                                    as isize,
                                            ))
                                            .type_0
                                                as ::core::ffi::c_uint
                                                != kExprNodeUnknownFigure as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                        {
                                            ast_stack.size =
                                                ast_stack.size.wrapping_sub(1 as size_t);
                                        }
                                        '_c2rust_label_39: {
                                            if **ast_stack.items.offset(
                                                ast_stack
                                                    .size
                                                    .wrapping_sub(0 as size_t)
                                                    .wrapping_sub(1 as size_t)
                                                    as isize,
                                            ) == lambda_node
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"(*kv_last(ast_stack)) == lambda_node\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2676 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        let node__3: *mut ExprASTNode = lambda_node;
                                        '_c2rust_label_40: {
                                            if (*node__3).type_0 as ::core::ffi::c_uint
                                                == kExprNodeUnknownFigure as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || (*node__3).type_0 as ::core::ffi::c_uint
                                                    == kExprNodeLambda as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"node_->type == kExprNodeUnknownFigure || node_->type == kExprNodeLambda\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2677 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        (*node__3).type_0 = kExprNodeLambda;
                                        if !(*pstate).colors.is_null() {
                                            (*(*(*pstate).colors).items.offset(
                                                (*node__3).data.fig.opening_hl_idx as isize,
                                            ))
                                            .group = if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidLambda\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimLambda\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            };
                                        }
                                        cur_node = viml_pexpr_new_node(kExprNodeArrow);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        if (*lambda_node).children.is_null() {
                                            '_c2rust_label_41: {
                                                if want_node as ::core::ffi::c_uint
                                                    == kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"want_node == kENodeValue\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2680 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            (*lambda_node).children = cur_node;
                                            if ast_stack.size == ast_stack.capacity {
                                                ast_stack.capacity = if ast_stack.capacity
                                                    << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                                };
                                                ast_stack.items = (if ast_stack.capacity
                                                    == ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            ast_stack.items
                                                                as *const ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut *mut *mut ExprASTNode;
                                            } else {
                                            };
                                            let c2rust_fresh22 = ast_stack.size;
                                            ast_stack.size = ast_stack.size.wrapping_add(1);
                                            let c2rust_lvalue_ptr_10 = &raw mut *ast_stack
                                                .items
                                                .offset(c2rust_fresh22 as isize);
                                            *c2rust_lvalue_ptr_10 =
                                                &raw mut (*lambda_node).children;
                                        } else {
                                            '_c2rust_label_42: {
                                                if (*(*lambda_node).children).next.is_null() {
                                                } else {
                                                    __assert_fail(
                                                        b"lambda_node->children->next == NULL\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2684 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            (*(*lambda_node).children).next = cur_node;
                                            if ast_stack.size == ast_stack.capacity {
                                                ast_stack.capacity = if ast_stack.capacity
                                                    << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                                };
                                                ast_stack.items = (if ast_stack.capacity
                                                    == ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            ast_stack.items
                                                                as *const ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut *mut *mut ExprASTNode;
                                            } else {
                                            };
                                            let c2rust_fresh23 = ast_stack.size;
                                            ast_stack.size = ast_stack.size.wrapping_add(1);
                                            let c2rust_lvalue_ptr_11 = &raw mut *ast_stack
                                                .items
                                                .offset(c2rust_fresh23 as isize);
                                            *c2rust_lvalue_ptr_11 =
                                                &raw mut (*(*lambda_node).children).next;
                                        }
                                        if ast_stack.size == ast_stack.capacity {
                                            ast_stack.capacity =
                                                if ast_stack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                                };
                                            ast_stack.items =
                                                (if ast_stack.capacity
                                                    == ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            ast_stack.items
                                                                as *const ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut *mut *mut ExprASTNode;
                                        } else {
                                        };
                                        let c2rust_fresh24 = ast_stack.size;
                                        ast_stack.size = ast_stack.size.wrapping_add(1);
                                        let c2rust_lvalue_ptr_12 = &raw mut *ast_stack
                                            .items
                                            .offset(c2rust_fresh24 as isize);
                                        *c2rust_lvalue_ptr_12 = &raw mut (*cur_node).children;
                                        lambda_node = ::core::ptr::null_mut::<ExprASTNode>();
                                    } else {
                                        if want_node as ::core::ffi::c_uint
                                            == kENodeValue as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(b"E15: Unexpected arrow: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                cur_token.start,
                                            );
                                            *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                            (**top_node_p).start = cur_token.start;
                                            (**top_node_p).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (**top_node_p).start = prev_token.start;
                                                (**top_node_p).len =
                                                    (**top_node_p).len.wrapping_add(prev_token.len);
                                            }
                                            (**top_node_p).len = 0 as size_t;
                                            want_node = kENodeOperator;
                                        }
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Arrow outside of lambda: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeArrow);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    }
                                    want_node = kENodeValue;
                                    viml_parser_highlight(
                                        pstate,
                                        cur_token.start,
                                        cur_token.len,
                                        if is_invalid as ::core::ffi::c_int != 0 {
                                            b"NvimInvalidArrow\0".as_ptr()
                                                as *const ::core::ffi::c_char
                                        } else {
                                            b"NvimArrow\0".as_ptr() as *const ::core::ffi::c_char
                                        },
                                    );
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                20 => {
                                    let scope: ExprVarScope =
                                        (if cur_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexInvalid as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            kExprVarScopeMissing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        } else {
                                            cur_token.data.var.scope as ::core::ffi::c_uint
                                        }) as ExprVarScope;
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        want_node = kENodeOperator;
                                        cur_node = viml_pexpr_new_node(
                                            (if node_is_key as ::core::ffi::c_int != 0 {
                                                kExprNodePlainKey as ::core::ffi::c_int
                                            } else {
                                                kExprNodePlainIdentifier as ::core::ffi::c_int
                                            })
                                                as ExprASTNodeType,
                                        );
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).data.var.scope = scope;
                                        let scope_shift_0: size_t = (if scope as ::core::ffi::c_uint
                                            == kExprVarScopeMissing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            0 as ::core::ffi::c_int
                                        } else {
                                            2 as ::core::ffi::c_int
                                        })
                                            as size_t;
                                        (*cur_node).data.var.ident = pline
                                            .data
                                            .offset(cur_token.start.col as isize)
                                            .offset(scope_shift_0 as isize);
                                        (*cur_node).data.var.ident_len =
                                            cur_token.len.wrapping_sub(scope_shift_0);
                                        *top_node_p = cur_node;
                                        if scope_shift_0 != 0 {
                                            '_c2rust_label_43: {
                                                if !node_is_key {
                                                } else {
                                                    __assert_fail(
                                                        b"!node_is_key\0".as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2717 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                1 as size_t,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidIdentifierScope\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimIdentifierScope\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                shifted_pos(cur_token.start, 1 as size_t),
                                                1 as size_t,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidIdentifierScopeDelimiter\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimIdentifierScopeDelimiter\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        viml_parser_highlight(
                                            pstate,
                                            shifted_pos(cur_token.start, scope_shift_0),
                                            cur_token.len.wrapping_sub(scope_shift_0),
                                            if node_is_key as ::core::ffi::c_int != 0 {
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidIdentifierKey\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimIdentifierKey\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                }
                                            } else if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidIdentifierName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimIdentifierName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    } else if scope as ::core::ffi::c_uint
                                        == kExprVarScopeMissing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        '_c2rust_label_44: {
                                            if want_node as ::core::ffi::c_uint
                                                == kENodeOperator as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"want_node == kENodeOperator\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2739 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                                && ast_stack.size == 1 as size_t
                                            {
                                                break '_viml_pexpr_parse_end;
                                            }
                                            '_c2rust_label_45: {
                                                if !(*top_node_p).is_null() {
                                                } else {
                                                    __assert_fail(
                                                        b"*top_node_p != NULL\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2739 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                cur_token.start,
                                            );
                                            cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).len = 0 as size_t;
                                            is_invalid = is_invalid as ::core::ffi::c_int
                                                | !viml_pexpr_handle_bop(
                                                    pstate,
                                                    &raw mut ast_stack,
                                                    cur_node,
                                                    &raw mut want_node,
                                                    &raw mut ast.err,
                                                )
                                                    as ::core::ffi::c_int
                                                != 0;
                                        } else {
                                            match (**top_node_p).type_0 as ::core::ffi::c_uint {
                                                13 | 11 | 17 => {
                                                    cur_node = viml_pexpr_new_node(
                                                        kExprNodeComplexIdentifier,
                                                    );
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    (*cur_node).len = 0 as size_t;
                                                    (*cur_node).children = *top_node_p;
                                                    *top_node_p = cur_node;
                                                    if ast_stack.size == ast_stack.capacity {
                                                        ast_stack.capacity = if ast_stack.capacity
                                                            << 1 as ::core::ffi::c_int
                                                            > ::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [*mut *mut ExprASTNode; 16],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            ) {
                                                            ast_stack.capacity
                                                                << 1 as ::core::ffi::c_int
                                                        } else {
                                                            ::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [*mut *mut ExprASTNode; 16],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as size_t,
                                                            )
                                                        };
                                                        ast_stack.items = (if ast_stack.capacity
                                                            == ::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [*mut *mut ExprASTNode; 16],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            ) {
                                                            if ast_stack.items
                                                                == &raw mut ast_stack.init_array
                                                                    as *mut *mut *mut ExprASTNode
                                                            {
                                                                ast_stack.items
                                                                    as *mut ::core::ffi::c_void
                                                            } else {
                                                                _memcpy_free(
                                                                    &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                                                                        as *mut ::core::ffi::c_void,
                                                                    ast_stack.items as *mut ::core::ffi::c_void,
                                                                    ast_stack
                                                                        .size
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                        ),
                                                                )
                                                            }
                                                        } else {
                                                            if ast_stack.items
                                                                == &raw mut ast_stack.init_array
                                                                    as *mut *mut *mut ExprASTNode
                                                            {
                                                                memcpy(
                                                                    xmalloc(
                                                                        ast_stack
                                                                            .capacity
                                                                            .wrapping_mul(
                                                                                ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                            ),
                                                                    ),
                                                                    ast_stack.items as *const ::core::ffi::c_void,
                                                                    ast_stack
                                                                        .size
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                                        ),
                                                                )
                                                            } else {
                                                                xrealloc(
                                                                    ast_stack.items
                                                                        as *mut ::core::ffi::c_void,
                                                                    ast_stack
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            *mut *mut ExprASTNode,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                )
                                                            }
                                                        })
                                                            as *mut *mut *mut ExprASTNode;
                                                    } else {
                                                    };
                                                    let c2rust_fresh25 = ast_stack.size;
                                                    ast_stack.size = ast_stack.size.wrapping_add(1);
                                                    let c2rust_lvalue_ptr_13 = &raw mut *ast_stack
                                                        .items
                                                        .offset(c2rust_fresh25 as isize);
                                                    *c2rust_lvalue_ptr_13 =
                                                        &raw mut (*(*cur_node).children).next;
                                                    let new_top_node_p_2: *mut *mut ExprASTNode =
                                                        *ast_stack.items.offset(
                                                            ast_stack
                                                                .size
                                                                .wrapping_sub(0 as size_t)
                                                                .wrapping_sub(1 as size_t)
                                                                as isize,
                                                        );
                                                    '_c2rust_label_46: {
                                                        if (*new_top_node_p_2).is_null() {
                                                        } else {
                                                            __assert_fail(
                                                                b"*new_top_node_p == NULL\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                                b"src/nvim/viml/parser/expressions.rs\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                                2739 as ::core::ffi::c_uint,
                                                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                            );
                                                        }
                                                    };
                                                    cur_node = viml_pexpr_new_node(
                                                        kExprNodePlainIdentifier,
                                                    );
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    (*cur_node).data.var.scope = scope;
                                                    (*cur_node).data.var.ident = pline
                                                        .data
                                                        .offset(cur_token.start.col as isize);
                                                    (*cur_node).data.var.ident_len = cur_token.len;
                                                    want_node = kENodeOperator;
                                                    *new_top_node_p_2 = cur_node;
                                                    viml_parser_highlight(
                                                        pstate,
                                                        cur_token.start,
                                                        cur_token.len,
                                                        if is_invalid as ::core::ffi::c_int != 0 {
                                                            b"NvimInvalidIdentifierName\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                        } else {
                                                            b"NvimIdentifierName\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                        },
                                                    );
                                                    break '_viml_pexpr_parse_cycle_end;
                                                }
                                                _ => {
                                                    if flags & kExprFlagsMulti as ::core::ffi::c_int
                                                        != 0
                                                        && ast_stack.size == 1 as size_t
                                                    {
                                                        break '_viml_pexpr_parse_end;
                                                    }
                                                    '_c2rust_label_47: {
                                                        if !(*top_node_p).is_null() {
                                                        } else {
                                                            __assert_fail(
                                                                b"*top_node_p != NULL\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                                b"src/nvim/viml/parser/expressions.rs\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                                2739 as ::core::ffi::c_uint,
                                                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                            );
                                                        }
                                                    };
                                                    is_invalid = true_0 != 0;
                                                    east_set_error(
                                                        pstate,
                                                        &raw mut ast.err,
                                                        gettext(
                                                            b"E15: Missing operator: %.*s\0"
                                                                .as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                        cur_token.start,
                                                    );
                                                    cur_node =
                                                        viml_pexpr_new_node(kExprNodeOpMissing);
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    (*cur_node).len = 0 as size_t;
                                                    is_invalid = is_invalid as ::core::ffi::c_int
                                                        | !viml_pexpr_handle_bop(
                                                            pstate,
                                                            &raw mut ast_stack,
                                                            cur_node,
                                                            &raw mut want_node,
                                                            &raw mut ast.err,
                                                        )
                                                            as ::core::ffi::c_int
                                                        != 0;
                                                }
                                            }
                                        }
                                    } else {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.size == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        '_c2rust_label_48: {
                                            if !(*top_node_p).is_null() {
                                            } else {
                                                __assert_fail(
                                                    b"*top_node_p != NULL\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2742 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    }
                                }
                                14 => {
                                    if want_node as ::core::ffi::c_uint
                                        != kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.size == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        '_c2rust_label_49: {
                                            if !(*top_node_p).is_null() {
                                            } else {
                                                __assert_fail(
                                                    b"*top_node_p != NULL\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2749 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        if node_is_key {
                                            cur_node = viml_pexpr_new_node(kExprNodePlainKey);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).data.var.ident =
                                                pline.data.offset(cur_token.start.col as isize);
                                            (*cur_node).data.var.ident_len = cur_token.len;
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidIdentifierKey\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimIdentifierKey\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        } else if cur_token.data.num.is_float {
                                            cur_node = viml_pexpr_new_node(kExprNodeFloat);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).data.flt.value =
                                                cur_token.data.num.val.floating;
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidFloat\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimFloat\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        } else {
                                            cur_node = viml_pexpr_new_node(kExprNodeInteger);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).data.num.value =
                                                cur_token.data.num.val.integer;
                                            let prefix_length: uint8_t = (*base_to_prefix_length
                                                .ptr())
                                                [cur_token.data.num.base as usize];
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                prefix_length as size_t,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidNumberPrefix\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimNumberPrefix\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                shifted_pos(
                                                    cur_token.start,
                                                    prefix_length as size_t,
                                                ),
                                                cur_token.len.wrapping_sub(prefix_length as size_t),
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidNumber\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimNumber\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        want_node = kENodeOperator;
                                        *top_node_p = cur_node;
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                11 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Unexpected dot: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if cur_pt as ::core::ffi::c_uint
                                            == kEPTAssignment as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Cannot concatenate in assignments: %.*s\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                        }
                                        cur_node = viml_pexpr_new_node(kExprNodeConcat);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidConcat\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimConcat\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeConcatOrSubscript);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidConcatOrSubscript\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimConcatOrSubscript\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &raw mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                23 => {
                                    if cur_token.data.brc.closing {
                                        's_5886: {
                                            if want_node as ::core::ffi::c_uint
                                                == kENodeValue as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                if ast_stack.size > 1 as size_t {
                                                    let prev_top_node: *const ExprASTNode =
                                                        **ast_stack.items.offset(
                                                            ast_stack
                                                                .size
                                                                .wrapping_sub(1 as size_t)
                                                                .wrapping_sub(1 as size_t)
                                                                as isize,
                                                        );
                                                    if (*prev_top_node).type_0
                                                        as ::core::ffi::c_uint
                                                        == kExprNodeCall as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        ast_stack.size = ast_stack
                                                            .size
                                                            .wrapping_sub(1 as size_t);
                                                        break 's_5886;
                                                    }
                                                }
                                                is_invalid = true_0 != 0;
                                                east_set_error(
                                                    pstate,
                                                    &raw mut ast.err,
                                                    gettext(
                                                        b"E15: Expected value, got parenthesis: %.*s\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ),
                                                    cur_token.start,
                                                );
                                                cur_node = viml_pexpr_new_node(kExprNodeMissing);
                                                (*cur_node).start = cur_token.start;
                                                (*cur_node).len = cur_token.len;
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    == kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).start = prev_token.start;
                                                    (*cur_node).len = (*cur_node)
                                                        .len
                                                        .wrapping_add(prev_token.len);
                                                }
                                                (*cur_node).len = 0 as size_t;
                                                *top_node_p = cur_node;
                                            } else {
                                                ast_stack.size =
                                                    ast_stack.size.wrapping_sub(1 as size_t);
                                            }
                                        }
                                        let mut new_top_node_p_3: *mut *mut ExprASTNode =
                                            ::core::ptr::null_mut::<*mut ExprASTNode>();
                                        while ast_stack.size != 0
                                            && (new_top_node_p_3.is_null()
                                                || (**new_top_node_p_3).type_0
                                                    as ::core::ffi::c_uint
                                                    != kExprNodeNested as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                    && (**new_top_node_p_3).type_0
                                                        as ::core::ffi::c_uint
                                                        != kExprNodeCall as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint)
                                        {
                                            ast_stack.size = ast_stack.size.wrapping_sub(1);
                                            new_top_node_p_3 =
                                                *ast_stack.items.offset(ast_stack.size as isize);
                                        }
                                        if !new_top_node_p_3.is_null()
                                            && ((**new_top_node_p_3).type_0 as ::core::ffi::c_uint
                                                == kExprNodeNested as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || (**new_top_node_p_3).type_0
                                                    as ::core::ffi::c_uint
                                                    == kExprNodeCall as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint)
                                        {
                                            if (**new_top_node_p_3).type_0 as ::core::ffi::c_uint
                                                == kExprNodeNested as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                viml_parser_highlight(
                                                    pstate,
                                                    cur_token.start,
                                                    cur_token.len,
                                                    if is_invalid as ::core::ffi::c_int != 0 {
                                                        b"NvimInvalidNestingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    } else {
                                                        b"NvimNestingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    },
                                                );
                                            } else {
                                                viml_parser_highlight(
                                                    pstate,
                                                    cur_token.start,
                                                    cur_token.len,
                                                    if is_invalid as ::core::ffi::c_int != 0 {
                                                        b"NvimInvalidCallingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    } else {
                                                        b"NvimCallingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    },
                                                );
                                            }
                                        } else {
                                            if new_top_node_p_3.is_null() {
                                                new_top_node_p_3 = top_node_p;
                                            }
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Unexpected closing parenthesis: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidNestingParenthesis\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimNestingParenthesis\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                            cur_node = viml_pexpr_new_node(kExprNodeNested);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = 0 as size_t;
                                            (*cur_node).children = *new_top_node_p_3;
                                            *new_top_node_p_3 = cur_node;
                                            '_c2rust_label_50: {
                                                if (*cur_node).next.is_null() {
                                                } else {
                                                    __assert_fail(
                                                        b"cur_node->next == NULL\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/viml/parser/expressions.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        2841 as ::core::ffi::c_uint,
                                                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                        }
                                        if ast_stack.size == ast_stack.capacity {
                                            ast_stack.capacity =
                                                if ast_stack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    ast_stack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                                };
                                            ast_stack.items =
                                                (if ast_stack.capacity
                                                    == ::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_div(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        ast_stack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if ast_stack.items
                                                        == &raw mut ast_stack.init_array
                                                            as *mut *mut *mut ExprASTNode
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            ast_stack.items
                                                                as *const ::core::ffi::c_void,
                                                            ast_stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            ast_stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    *mut *mut ExprASTNode,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut *mut *mut ExprASTNode;
                                        } else {
                                        };
                                        let c2rust_fresh26 = ast_stack.size;
                                        ast_stack.size = ast_stack.size.wrapping_add(1);
                                        let c2rust_lvalue_ptr_14 = &raw mut *ast_stack
                                            .items
                                            .offset(c2rust_fresh26 as isize);
                                        *c2rust_lvalue_ptr_14 = new_top_node_p_3;
                                        want_node = kENodeOperator;
                                        break '_viml_pexpr_parse_cycle_end;
                                    } else {
                                        match want_node as ::core::ffi::c_uint {
                                            1 => {
                                                cur_node = viml_pexpr_new_node(kExprNodeNested);
                                                (*cur_node).start = cur_token.start;
                                                (*cur_node).len = cur_token.len;
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    == kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).start = prev_token.start;
                                                    (*cur_node).len = (*cur_node)
                                                        .len
                                                        .wrapping_add(prev_token.len);
                                                }
                                                *top_node_p = cur_node;
                                                if ast_stack.size == ast_stack.capacity {
                                                    ast_stack.capacity = if ast_stack.capacity
                                                        << 1 as ::core::ffi::c_int
                                                        > ::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_div(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                        ast_stack.capacity
                                                            << 1 as ::core::ffi::c_int
                                                    } else {
                                                        ::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_div(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                    };
                                                    ast_stack.items = (if ast_stack.capacity
                                                        == ::core::mem::size_of::<
                                                            [*mut *mut ExprASTNode; 16],
                                                        >(
                                                        )
                                                        .wrapping_div(::core::mem::size_of::<
                                                            *mut *mut ExprASTNode,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [*mut *mut ExprASTNode; 16],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                *mut *mut ExprASTNode,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                        if ast_stack.items
                                                            == &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                        {
                                                            ast_stack.items
                                                                as *mut ::core::ffi::c_void
                                                        } else {
                                                            _memcpy_free(
                                                                &raw mut ast_stack.init_array
                                                                    as *mut *mut *mut ExprASTNode
                                                                    as *mut ::core::ffi::c_void,
                                                                ast_stack.items
                                                                    as *mut ::core::ffi::c_void,
                                                                ast_stack.size.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        }
                                                    } else {
                                                        if ast_stack.items
                                                            == &raw mut ast_stack.init_array
                                                                as *mut *mut *mut ExprASTNode
                                                        {
                                                            memcpy(
                                                                xmalloc(
                                                                    ast_stack
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            *mut *mut ExprASTNode,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                ),
                                                                ast_stack.items
                                                                    as *const ::core::ffi::c_void,
                                                                ast_stack.size.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        } else {
                                                            xrealloc(
                                                                ast_stack.items
                                                                    as *mut ::core::ffi::c_void,
                                                                ast_stack.capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        *mut *mut ExprASTNode,
                                                                    >(
                                                                    ),
                                                                ),
                                                            )
                                                        }
                                                    })
                                                        as *mut *mut *mut ExprASTNode;
                                                } else {
                                                };
                                                let c2rust_fresh27 = ast_stack.size;
                                                ast_stack.size = ast_stack.size.wrapping_add(1);
                                                let c2rust_lvalue_ptr_15 = &raw mut *ast_stack
                                                    .items
                                                    .offset(c2rust_fresh27 as isize);
                                                *c2rust_lvalue_ptr_15 =
                                                    &raw mut (*cur_node).children;
                                                viml_parser_highlight(
                                                    pstate,
                                                    cur_token.start,
                                                    cur_token.len,
                                                    if is_invalid as ::core::ffi::c_int != 0 {
                                                        b"NvimInvalidNestingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    } else {
                                                        b"NvimNestingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    },
                                                );
                                                break 's_6212;
                                            }
                                            0 => {
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    != kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    break;
                                                }
                                                if !((**top_node_p).type_0 as ::core::ffi::c_uint
                                                    != kExprNodePlainIdentifier
                                                        as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                    && (**top_node_p).type_0 as ::core::ffi::c_uint
                                                        != kExprNodeComplexIdentifier
                                                            as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    && (**top_node_p).type_0 as ::core::ffi::c_uint
                                                        != kExprNodeCurlyBracesIdentifier
                                                            as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint)
                                                {
                                                    break;
                                                }
                                                if flags & kExprFlagsMulti as ::core::ffi::c_int
                                                    != 0
                                                    && ast_stack.size == 1 as size_t
                                                {
                                                    break '_viml_pexpr_parse_end;
                                                }
                                                '_c2rust_label_51: {
                                                    if !(*top_node_p).is_null() {
                                                    } else {
                                                        __assert_fail(
                                                            b"*top_node_p != NULL\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                            b"src/nvim/viml/parser/expressions.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            2863 as ::core::ffi::c_uint,
                                                            b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                    }
                                                };
                                                is_invalid = true_0 != 0;
                                                east_set_error(
                                                    pstate,
                                                    &raw mut ast.err,
                                                    gettext(
                                                        b"E15: Missing operator: %.*s\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ),
                                                    cur_token.start,
                                                );
                                                cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                                (*cur_node).start = cur_token.start;
                                                (*cur_node).len = cur_token.len;
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    == kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).start = prev_token.start;
                                                    (*cur_node).len = (*cur_node)
                                                        .len
                                                        .wrapping_add(prev_token.len);
                                                }
                                                (*cur_node).len = 0 as size_t;
                                                is_invalid = is_invalid as ::core::ffi::c_int
                                                    | !viml_pexpr_handle_bop(
                                                        pstate,
                                                        &raw mut ast_stack,
                                                        cur_node,
                                                        &raw mut want_node,
                                                        &raw mut ast.err,
                                                    )
                                                        as ::core::ffi::c_int
                                                    != 0;
                                            }
                                            _ => {
                                                break 's_6212;
                                            }
                                        }
                                    }
                                }
                                4 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Expected value, got question mark: %.*s\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeTernary);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &raw mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    viml_parser_highlight(
                                        pstate,
                                        cur_token.start,
                                        cur_token.len,
                                        if is_invalid as ::core::ffi::c_int != 0 {
                                            b"NvimInvalidTernary\0".as_ptr()
                                                as *const ::core::ffi::c_char
                                        } else {
                                            b"NvimTernary\0".as_ptr() as *const ::core::ffi::c_char
                                        },
                                    );
                                    let mut ter_val_node: *mut ExprASTNode =
                                        ::core::ptr::null_mut::<ExprASTNode>();
                                    ter_val_node = viml_pexpr_new_node(kExprNodeTernaryValue);
                                    (*ter_val_node).start = cur_token.start;
                                    (*ter_val_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*ter_val_node).start = prev_token.start;
                                        (*ter_val_node).len =
                                            (*ter_val_node).len.wrapping_add(prev_token.len);
                                    }
                                    (*ter_val_node).data.ter.got_colon = false_0 != 0;
                                    '_c2rust_label_52: {
                                        if !(*cur_node).children.is_null() {
                                        } else {
                                            __assert_fail(
                                                b"cur_node->children != NULL\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"src/nvim/viml/parser/expressions.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                2882 as ::core::ffi::c_uint,
                                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    '_c2rust_label_53: {
                                        if (*(*cur_node).children).next.is_null() {
                                        } else {
                                            __assert_fail(
                                                b"cur_node->children->next == NULL\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"src/nvim/viml/parser/expressions.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                2883 as ::core::ffi::c_uint,
                                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    '_c2rust_label_54: {
                                        if *ast_stack.items.offset(
                                            ast_stack
                                                .size
                                                .wrapping_sub(0 as size_t)
                                                .wrapping_sub(1 as size_t)
                                                as isize,
                                        ) == &raw mut (*(*cur_node).children).next
                                        {
                                        } else {
                                            __assert_fail(
                                                b"kv_last(ast_stack) == &cur_node->children->next\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                b"src/nvim/viml/parser/expressions.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                2884 as ::core::ffi::c_uint,
                                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    **ast_stack.items.offset(
                                        ast_stack
                                            .size
                                            .wrapping_sub(0 as size_t)
                                            .wrapping_sub(1 as size_t)
                                            as isize,
                                    ) = ter_val_node;
                                    if ast_stack.size == ast_stack.capacity {
                                        ast_stack.capacity = if ast_stack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                                .wrapping_div(::core::mem::size_of::<
                                                    *mut *mut ExprASTNode,
                                                >(
                                                ))
                                                .wrapping_div(
                                                    (::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    )) == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            ast_stack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                                .wrapping_div(::core::mem::size_of::<
                                                    *mut *mut ExprASTNode,
                                                >(
                                                ))
                                                .wrapping_div(
                                                    (::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    )) == 0)
                                                        as ::core::ffi::c_int
                                                        as size_t,
                                                )
                                        };
                                        ast_stack.items = (if ast_stack.capacity
                                            == ::core::mem::size_of::<[*mut *mut ExprASTNode; 16]>()
                                                .wrapping_div(::core::mem::size_of::<
                                                    *mut *mut ExprASTNode,
                                                >(
                                                ))
                                                .wrapping_div(
                                                    (::core::mem::size_of::<
                                                        [*mut *mut ExprASTNode; 16],
                                                    >(
                                                    )
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        *mut *mut ExprASTNode,
                                                    >(
                                                    )) == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            if ast_stack.items
                                                == &raw mut ast_stack.init_array
                                                    as *mut *mut *mut ExprASTNode
                                            {
                                                ast_stack.items as *mut ::core::ffi::c_void
                                            } else {
                                                _memcpy_free(
                                                    &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode
                                                        as *mut ::core::ffi::c_void,
                                                    ast_stack.items as *mut ::core::ffi::c_void,
                                                    ast_stack
                                                        .size
                                                        .wrapping_mul(
                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                        ),
                                                )
                                            }
                                        } else {
                                            if ast_stack.items
                                                == &raw mut ast_stack.init_array
                                                    as *mut *mut *mut ExprASTNode
                                            {
                                                memcpy(
                                                    xmalloc(
                                                        ast_stack
                                                            .capacity
                                                            .wrapping_mul(
                                                                ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                            ),
                                                    ),
                                                    ast_stack.items as *const ::core::ffi::c_void,
                                                    ast_stack
                                                        .size
                                                        .wrapping_mul(
                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                        ),
                                                )
                                            } else {
                                                xrealloc(
                                                    ast_stack.items as *mut ::core::ffi::c_void,
                                                    ast_stack
                                                        .capacity
                                                        .wrapping_mul(
                                                            ::core::mem::size_of::<*mut *mut ExprASTNode>(),
                                                        ),
                                                )
                                            }
                                        })
                                            as *mut *mut *mut ExprASTNode;
                                    } else {
                                    };
                                    let c2rust_fresh28 = ast_stack.size;
                                    ast_stack.size = ast_stack.size.wrapping_add(1);
                                    let c2rust_lvalue_ptr_16 =
                                        &raw mut *ast_stack.items.offset(c2rust_fresh28 as isize);
                                    *c2rust_lvalue_ptr_16 = &raw mut (*ter_val_node).children;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                16 | 15 => {
                                    let is_double: bool = tok_type as ::core::ffi::c_uint
                                        == kExprLexDoubleQuotedString as ::core::ffi::c_int
                                            as ::core::ffi::c_uint;
                                    if !cur_token.data.str.closed {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            if is_double as ::core::ffi::c_int != 0 {
                                                gettext(
                                                    b"E114: Missing double quote: %.*s\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                )
                                            } else {
                                                gettext(
                                                    b"E115: Missing single quote: %.*s\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                )
                                            },
                                            cur_token.start,
                                        );
                                    }
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.size == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        '_c2rust_label_55: {
                                            if !(*top_node_p).is_null() {
                                            } else {
                                                __assert_fail(
                                                    b"*top_node_p != NULL\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/viml/parser/expressions.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    2901 as ::core::ffi::c_uint,
                                                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &raw mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(
                                            (if is_double as ::core::ffi::c_int != 0 {
                                                kExprNodeDoubleQuotedString as ::core::ffi::c_int
                                            } else {
                                                kExprNodeSingleQuotedString as ::core::ffi::c_int
                                            })
                                                as ExprASTNodeType,
                                        );
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        *top_node_p = cur_node;
                                        parse_quoted_string(
                                            pstate,
                                            cur_node,
                                            cur_token,
                                            &raw mut ast_stack,
                                            is_invalid,
                                        );
                                        want_node = kENodeOperator;
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                26 => {
                                    if cur_pt as ::core::ffi::c_uint
                                        == kEPTAssignment as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        pt_stack.size = pt_stack.size.wrapping_sub(1 as size_t);
                                    } else if cur_pt as ::core::ffi::c_uint
                                        == kEPTSingleAssignment as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        pt_stack.size = pt_stack.size.wrapping_sub(2 as size_t);
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E475: Expected closing bracket to end list assignment lvalue: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                    } else {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Misplaced assignment: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                    }
                                    '_c2rust_label_56: {
                                        if pt_stack.size != 0 {
                                        } else {
                                            __assert_fail(
                                                b"kv_size(pt_stack)\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"src/nvim/viml/parser/expressions.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                2922 as ::core::ffi::c_uint,
                                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    '_c2rust_label_57: {
                                        if *pt_stack.items.offset(
                                            pt_stack
                                                .size
                                                .wrapping_sub(0 as size_t)
                                                .wrapping_sub(1 as size_t)
                                                as isize,
                                        )
                                            as ::core::ffi::c_uint
                                            == kEPTExpr as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                        } else {
                                            __assert_fail(
                                                b"kv_last(pt_stack) == kEPTExpr\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"src/nvim/viml/parser/expressions.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                2923 as ::core::ffi::c_uint,
                                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Unexpected assignment: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeAssignment);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    (*cur_node).data.ass.type_0 = cur_token.data.ass.type_0;
                                    match cur_token.data.ass.type_0 as ::core::ffi::c_uint {
                                        0 => {
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidPlainAssignment\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimPlainAssignment\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        1 => {
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidAssignmentWithAddition\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimAssignmentWithAddition\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        2 => {
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidAssignmentWithSubtraction\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimAssignmentWithSubtraction\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        3 => {
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidAssignmentWithConcatenation\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimAssignmentWithConcatenation\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        _ => {}
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &raw mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                _ => {
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                            }
                        }
                    }
                    cur_node = viml_pexpr_new_node(kExprNodeCall);
                    (*cur_node).start = cur_token.start;
                    (*cur_node).len = cur_token.len;
                    if prev_token.type_0 as ::core::ffi::c_uint
                        == kExprLexSpacing as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        (*cur_node).start = prev_token.start;
                        (*cur_node).len = (*cur_node).len.wrapping_add(prev_token.len);
                    }
                    is_invalid = is_invalid as ::core::ffi::c_int
                        | !viml_pexpr_handle_bop(
                            pstate,
                            &raw mut ast_stack,
                            cur_node,
                            &raw mut want_node,
                            &raw mut ast.err,
                        ) as ::core::ffi::c_int
                        != 0;
                    viml_parser_highlight(
                        pstate,
                        cur_token.start,
                        cur_token.len,
                        if is_invalid as ::core::ffi::c_int != 0 {
                            b"NvimInvalidCallingParenthesis\0".as_ptr()
                                as *const ::core::ffi::c_char
                        } else {
                            b"NvimCallingParenthesis\0".as_ptr() as *const ::core::ffi::c_char
                        },
                    );
                    break 's_6212;
                }
                if pt_is_assignment(cur_pt) as ::core::ffi::c_int != 0
                    && !pt_is_assignment(
                        *pt_stack.items.offset(
                            pt_stack
                                .size
                                .wrapping_sub(0 as size_t)
                                .wrapping_sub(1 as size_t) as isize,
                        ),
                    )
                {
                    '_c2rust_label_36: {
                        if want_node as ::core::ffi::c_uint
                            == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                        } else {
                            __assert_fail(
                                b"want_node == kENodeValue\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/viml/parser/expressions.rs\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                2657 as ::core::ffi::c_uint,
                                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    asgn_level = ast_stack.size.wrapping_sub(1 as size_t);
                }
                break '_viml_pexpr_parse_cycle_end;
            }
            want_node = kENodeValue;
        }
        prev_token = cur_token;
        highlighted_prev_spacing = false_0 != 0;
        viml_parser_advance(pstate, cur_token.len);
    }
    '_c2rust_label_58: {
        if pt_stack.size != 0 {
        } else {
            __assert_fail(
                b"kv_size(pt_stack)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/viml/parser/expressions.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2945 as ::core::ffi::c_uint,
                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_59: {
        if ast_stack.size != 0 {
        } else {
            __assert_fail(
                b"kv_size(ast_stack)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/viml/parser/expressions.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2946 as ::core::ffi::c_uint,
                b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if want_node as ::core::ffi::c_uint == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
        && *pt_stack.items.offset(
            pt_stack
                .size
                .wrapping_sub(0 as size_t)
                .wrapping_sub(1 as size_t) as isize,
        ) as ::core::ffi::c_uint
            != kEPTLambdaArguments as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        east_set_error(
            pstate,
            &raw mut ast.err,
            gettext(b"E15: Expected value, got EOC: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            (*pstate).pos,
        );
    } else if ast_stack.size != 1 as size_t {
        '_c2rust_label_60: {
            if ast_stack.size != 0 {
            } else {
                __assert_fail(
                    b"kv_size(ast_stack)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/viml/parser/expressions.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2958 as ::core::ffi::c_uint,
                    b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        ast_stack.size = ast_stack.size.wrapping_sub(1 as size_t);
        while ast.err.msg.is_null() && ast_stack.size != 0 {
            ast_stack.size = ast_stack.size.wrapping_sub(1);
            let cur_node_0: *const ExprASTNode = **ast_stack.items.offset(ast_stack.size as isize);
            '_c2rust_label_61: {
                if !cur_node_0.is_null() {
                } else {
                    __assert_fail(
                        b"cur_node != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/viml/parser/expressions.rs\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        2965 as ::core::ffi::c_uint,
                        b"ExprAST viml_pexpr_parse(ParserState *const, const int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            match (*cur_node_0).type_0 as ::core::ffi::c_uint {
                10 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(
                            b"E116: Missing closing parenthesis for function call: %.*s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        (*cur_node_0).start,
                    );
                }
                9 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(
                            b"E110: Missing closing parenthesis for nested expression: %.*s\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        (*cur_node_0).start,
                    );
                }
                6 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(b"E697: Missing end of List ']': %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*cur_node_0).start,
                    );
                }
                16 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(b"E723: Missing end of Dictionary '}': %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*cur_node_0).start,
                    );
                }
                14 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(b"E15: Missing closing figure brace: %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*cur_node_0).start,
                    );
                }
                15 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(
                            b"E15: Missing closing figure brace for lambda: %.*s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        (*cur_node_0).start,
                    );
                }
                17 => {
                    abort();
                }
                24 | 25 | 26 | 27 | 36 | 37 | 4 | 11 | 12 => {
                    abort();
                }
                18 | 19 | 20 => {}
                5 | 23 | 13 | 38 | 35 | 34 | 33 | 32 | 29 | 28 | 22 | 21 | 30 | 7 | 31 | 2 | 8 => {}
                3 => {
                    if !(*cur_node_0).data.ter.got_colon {
                        east_set_error(
                            pstate,
                            &raw mut ast.err,
                            gettext(b"E109: Missing ':' after '?': %.*s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            (*cur_node_0).start,
                        );
                    }
                }
                1 | 0 | _ => {}
            }
        }
    }
    if ast_stack.items != &raw mut ast_stack.init_array as *mut *mut *mut ExprASTNode {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut ast_stack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    return ast;
}
#[inline(always)]
unsafe extern "C" fn viml_parser_advance(pstate: *mut ParserState, len: size_t) {
    '_c2rust_label: {
        if (*pstate).pos.line == (*pstate).reader.lines.size.wrapping_sub(1 as size_t) {
        } else {
            __assert_fail(
                b"pstate->pos.line == kv_size(pstate->reader.lines) - 1\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/viml/parser/expressions.rs\0".as_ptr() as *const ::core::ffi::c_char,
                48 as ::core::ffi::c_uint,
                b"void viml_parser_advance(ParserState *const, const size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let pline: ParserLine = *(*pstate).reader.lines.items.offset(
        (*pstate)
            .reader
            .lines
            .size
            .wrapping_sub(0 as size_t)
            .wrapping_sub(1 as size_t) as isize,
    );
    if (*pstate).pos.col.wrapping_add(len) >= pline.size {
        (*pstate).pos.line = (*pstate).pos.line.wrapping_add(1);
        (*pstate).pos.col = 0 as size_t;
    } else {
        (*pstate).pos.col = (*pstate).pos.col.wrapping_add(len);
    };
}
#[inline(always)]
unsafe extern "C" fn viml_parser_highlight(
    pstate: *mut ParserState,
    start: ParserPosition,
    len: size_t,
    group: *const ::core::ffi::c_char,
) {
    if (*pstate).colors.is_null() || len == 0 as size_t {
        return;
    }
    '_c2rust_label: {
        if (*(*pstate).colors).size == 0 as size_t
            || (*(*(*pstate).colors).items.offset(
                (*(*pstate).colors)
                    .size
                    .wrapping_sub(0 as size_t)
                    .wrapping_sub(1 as size_t) as isize,
            ))
            .start
            .line
                < start.line
            || (*(*(*pstate).colors).items.offset(
                (*(*pstate).colors)
                    .size
                    .wrapping_sub(0 as size_t)
                    .wrapping_sub(1 as size_t) as isize,
            ))
            .end_col
                <= start.col
        {
        } else {
            __assert_fail(
                b"kv_size(*pstate->colors) == 0 || kv_Z(*pstate->colors, 0).start.line < start.line || kv_Z(*pstate->colors, 0).end_col <= start.col\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/viml/parser/expressions.rs\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                73 as ::core::ffi::c_uint,
                b"void viml_parser_highlight(ParserState *const, const ParserPosition, const size_t, const char *const)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if (*(*pstate).colors).size == (*(*pstate).colors).capacity {
        (*(*pstate).colors).capacity = if (*(*pstate).colors).capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[ParserHighlightChunk; 16]>()
                .wrapping_div(::core::mem::size_of::<ParserHighlightChunk>())
                .wrapping_div(
                    (::core::mem::size_of::<[ParserHighlightChunk; 16]>()
                        .wrapping_rem(::core::mem::size_of::<ParserHighlightChunk>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            (*(*pstate).colors).capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[ParserHighlightChunk; 16]>()
                .wrapping_div(::core::mem::size_of::<ParserHighlightChunk>())
                .wrapping_div(
                    (::core::mem::size_of::<[ParserHighlightChunk; 16]>()
                        .wrapping_rem(::core::mem::size_of::<ParserHighlightChunk>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        (*(*pstate).colors).items = (if (*(*pstate).colors).capacity
            == ::core::mem::size_of::<[ParserHighlightChunk; 16]>()
                .wrapping_div(::core::mem::size_of::<ParserHighlightChunk>())
                .wrapping_div(
                    (::core::mem::size_of::<[ParserHighlightChunk; 16]>()
                        .wrapping_rem(::core::mem::size_of::<ParserHighlightChunk>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if (*(*pstate).colors).items
                == &raw mut (*(*pstate).colors).init_array as *mut ParserHighlightChunk
            {
                (*(*pstate).colors).items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut (*(*pstate).colors).init_array as *mut ParserHighlightChunk
                        as *mut ::core::ffi::c_void,
                    (*(*pstate).colors).items as *mut ::core::ffi::c_void,
                    (*(*pstate).colors)
                        .size
                        .wrapping_mul(::core::mem::size_of::<ParserHighlightChunk>()),
                )
            }
        } else {
            if (*(*pstate).colors).items
                == &raw mut (*(*pstate).colors).init_array as *mut ParserHighlightChunk
            {
                memcpy(
                    xmalloc(
                        (*(*pstate).colors)
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<ParserHighlightChunk>()),
                    ),
                    (*(*pstate).colors).items as *const ::core::ffi::c_void,
                    (*(*pstate).colors)
                        .size
                        .wrapping_mul(::core::mem::size_of::<ParserHighlightChunk>()),
                )
            } else {
                xrealloc(
                    (*(*pstate).colors).items as *mut ::core::ffi::c_void,
                    (*(*pstate).colors)
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<ParserHighlightChunk>()),
                )
            }
        }) as *mut ParserHighlightChunk;
    } else {
    };
    let c2rust_fresh34 = (*(*pstate).colors).size;
    (*(*pstate).colors).size = (*(*pstate).colors).size.wrapping_add(1);
    *(*(*pstate).colors).items.offset(c2rust_fresh34 as isize) = ParserHighlightChunk {
        start: start,
        end_col: start.col.wrapping_add(len),
        group: group,
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
