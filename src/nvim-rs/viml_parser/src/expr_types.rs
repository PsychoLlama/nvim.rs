/// C-compatible type definitions for the VimL expression parser.
///
/// These types mirror the definitions in `src/nvim/viml/parser/expressions.h`
/// and must maintain ABI compatibility.  All structs and enums are `#[repr(C)]`
/// so that Rust implementations can be called directly from C callers.
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_uchar;

// ---------------------------------------------------------------------------
// Scalar types matching nvim's typedef aliases.
// ---------------------------------------------------------------------------

/// `float_T` from nvim/types_defs.h: typedef double float_T;
pub type FloatT = f64;

/// `uvarnumber_T` from nvim/eval/typval_defs.h: typedef uint64_t uvarnumber_T;
pub type UvarnumberT = u64;

// ---------------------------------------------------------------------------
// Enumerations
// ---------------------------------------------------------------------------

/// ExprCaseCompareStrategy — comparison case sensitivity strategy.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExprCaseCompareStrategy {
    /// `==` — use 'ignorecase' option.
    UseOption = 0,
    /// `==#` — match case.
    MatchCase = b'#' as isize,
    /// `==?` — ignore case.
    IgnoreCase = b'?' as isize,
}

/// LexExprTokenType — kind of a lexer token.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LexExprTokenType {
    Invalid = 0,
    Missing,
    Spacing,
    EOC,

    Question,
    Colon,
    Or,
    And,
    Comparison,
    Plus,
    Minus,
    Dot,
    Multiplication,

    Not,

    Number,
    SingleQuotedString,
    DoubleQuotedString,
    Option,
    Register,
    Env,
    PlainIdentifier,

    Bracket,
    FigureBrace,
    Parenthesis,
    Comma,
    Arrow,
    Assignment,
}

/// ExprComparisonType — which comparison operator.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExprComparisonType {
    Equal = 0,
    Matches,
    Greater,
    GreaterOrEqual,
    Identical,
}

/// ExprOptScope — option scope.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExprOptScope {
    Unspecified = 0,
    Global = b'g' as isize,
    Local = b'l' as isize,
}

impl TryFrom<u8> for ExprOptScope {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(ExprOptScope::Unspecified),
            b'g' => Ok(ExprOptScope::Global),
            b'l' => Ok(ExprOptScope::Local),
            other => Err(other),
        }
    }
}

/// ExprAssignmentType — assignment operator variant.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExprAssignmentType {
    Plain = 0,
    Add,
    Subtract,
    Concat,
}

/// ExprVarScope — variable scope character.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExprVarScope {
    Missing = 0,
    Script = b's' as isize,
    Global = b'g' as isize,
    Vim = b'v' as isize,
    Buffer = b'b' as isize,
    Window = b'w' as isize,
    Tabpage = b't' as isize,
    Local = b'l' as isize,
    Arguments = b'a' as isize,
}

/// LexExprFlags — bitmask flags for the lexer.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LexExprFlags(pub c_int);

impl LexExprFlags {
    pub const PEEK: c_int = 1 << 0;
    pub const FORBID_SCOPE: c_int = 1 << 1;
    pub const ALLOW_FLOAT: c_int = 1 << 2;
    pub const IS_NOT_CMP: c_int = 1 << 3;
    pub const FORBID_EOC: c_int = 1 << 4;
}

/// LexExprMulType — multiplication sub-kind.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LexExprMulType {
    Mul = 0,
    Div,
    Mod,
}

/// ExprASTNodeType — kind of an AST node.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExprASTNodeType {
    Missing = 0,
    OpMissing,
    Ternary,
    TernaryValue,
    Register,
    Subscript,
    ListLiteral,
    UnaryPlus,
    BinaryPlus,
    Nested,
    Call,
    PlainIdentifier,
    PlainKey,
    ComplexIdentifier,
    UnknownFigure,
    Lambda,
    DictLiteral,
    CurlyBracesIdentifier,
    Comma,
    Colon,
    Arrow,
    Comparison,
    Concat,
    ConcatOrSubscript,
    Integer,
    Float,
    SingleQuotedString,
    DoubleQuotedString,
    Or,
    And,
    UnaryMinus,
    BinaryMinus,
    Not,
    Multiplication,
    Division,
    Mod,
    Option,
    Environment,
    Assignment,
}

/// ExprParserFlags bitmask — parser-level flags.
pub const EXPR_FLAGS_MULTI: c_int = 1 << 0;
pub const EXPR_FLAGS_DISALLOW_EOC: c_int = 1 << 1;
pub const EXPR_FLAGS_PARSE_LET: c_int = 1 << 2;

// ---------------------------------------------------------------------------
// ParserPosition — mirrors parser_defs.h ParserPosition
// ---------------------------------------------------------------------------

/// Mirror of `ParserPosition` in `parser_defs.h`.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct ParserPosition {
    pub line: usize,
    pub col: usize,
}

// ---------------------------------------------------------------------------
// ParserLine — mirrors parser_defs.h ParserLine
// ---------------------------------------------------------------------------

/// Mirror of `ParserLine` in `parser_defs.h`.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ParserLine {
    pub data: *const c_char,
    pub size: usize,
    pub allocated: bool,
}

// ---------------------------------------------------------------------------
// LexExprToken union data fields
// ---------------------------------------------------------------------------

/// Data for kExprLexComparison.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LexTknDataCmp {
    pub typ: ExprComparisonType,
    pub ccs: ExprCaseCompareStrategy,
    pub inv: bool,
}

/// Data for kExprLexMultiplication.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LexTknDataMul {
    pub typ: LexExprMulType,
}

/// Data for brackets/braces/parentheses.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LexTknDataBrc {
    pub closing: bool,
}

/// Data for kExprLexRegister.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LexTknDataReg {
    pub name: c_int,
}

/// Data for single/double quoted strings.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LexTknDataStr {
    pub closed: bool,
}

/// Data for kExprLexOption.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LexTknDataOpt {
    pub name: *const c_char,
    pub len: usize,
    pub scope: ExprOptScope,
}

/// Data for kExprLexPlainIdentifier.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LexTknDataVar {
    pub scope: ExprVarScope,
    pub autoload: bool,
}

/// Data for kExprLexInvalid.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LexTknDataErr {
    pub typ: LexExprTokenType,
    pub msg: *const c_char,
}

/// Nested number value union: floating or integer.
#[repr(C)]
#[derive(Copy, Clone)]
pub union LexTknNumVal {
    pub floating: FloatT,
    pub integer: UvarnumberT,
}

/// Data for kExprLexNumber.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct LexTknDataNum {
    pub val: LexTknNumVal,
    pub base: c_uchar,
    pub is_float: bool,
}

/// Data for kExprLexAssignment.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LexTknDataAss {
    pub typ: ExprAssignmentType,
}

/// The union inside LexExprToken.
#[repr(C)]
#[derive(Copy, Clone)]
pub union LexTknData {
    pub cmp: LexTknDataCmp,
    pub mul: LexTknDataMul,
    pub brc: LexTknDataBrc,
    pub reg: LexTknDataReg,
    pub str_: LexTknDataStr,
    pub opt: LexTknDataOpt,
    pub var: LexTknDataVar,
    pub err: LexTknDataErr,
    pub num: LexTknDataNum,
    pub ass: LexTknDataAss,
}

/// LexExprToken — one lexer token.
///
/// Must match `LexExprToken` in expressions.h exactly.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct LexExprToken {
    pub start: ParserPosition,
    pub len: usize,
    pub typ: LexExprTokenType,
    pub data: LexTknData,
}

// ---------------------------------------------------------------------------
// ExprASTNode — AST node
// ---------------------------------------------------------------------------

/// Data for kExprNodeRegister.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstNodeDataReg {
    pub name: c_int,
}

/// Type guesses for UnknownFigure.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstFigTypeGuesses {
    pub allow_dict: bool,
    pub allow_lambda: bool,
    pub allow_ident: bool,
}

/// Data for kExprNodeUnknownFigure.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstNodeDataFig {
    pub type_guesses: AstFigTypeGuesses,
    pub opening_hl_idx: usize,
}

/// Data for kExprNodePlainIdentifier and kExprNodePlainKey.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstNodeDataVar {
    pub scope: ExprVarScope,
    pub ident: *const c_char,
    pub ident_len: usize,
}

/// Data for kExprNodeTernaryValue.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstNodeDataTer {
    pub got_colon: bool,
}

/// Data for kExprNodeComparison.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstNodeDataCmp {
    pub typ: ExprComparisonType,
    pub ccs: ExprCaseCompareStrategy,
    pub inv: bool,
}

/// Data for kExprNodeInteger.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstNodeDataNum {
    pub value: UvarnumberT,
}

/// Data for kExprNodeFloat.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstNodeDataFlt {
    pub value: FloatT,
}

/// Data for kExprNodeSingleQuotedString / kExprNodeDoubleQuotedString.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstNodeDataStr {
    pub value: *mut c_char,
    pub size: usize,
}

/// Data for kExprNodeOption.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstNodeDataOpt {
    pub ident: *const c_char,
    pub ident_len: usize,
    pub scope: ExprOptScope,
}

/// Data for kExprNodeEnvironment.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstNodeDataEnv {
    pub ident: *const c_char,
    pub ident_len: usize,
}

/// Data for kExprNodeAssignment.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AstNodeDataAss {
    pub typ: ExprAssignmentType,
}

/// The union inside ExprASTNode.
#[repr(C)]
#[derive(Copy, Clone)]
pub union AstNodeData {
    pub reg: AstNodeDataReg,
    pub fig: AstNodeDataFig,
    pub var: AstNodeDataVar,
    pub ter: AstNodeDataTer,
    pub cmp: AstNodeDataCmp,
    pub num: AstNodeDataNum,
    pub flt: AstNodeDataFlt,
    pub str_: AstNodeDataStr,
    pub opt: AstNodeDataOpt,
    pub env: AstNodeDataEnv,
    pub ass: AstNodeDataAss,
}

/// ExprASTNode — one node in the expression AST.
///
/// Must match `struct expr_ast_node` in expressions.h exactly.
#[repr(C)]
pub struct ExprASTNode {
    pub typ: ExprASTNodeType,
    pub children: *mut ExprASTNode,
    pub next: *mut ExprASTNode,
    pub start: ParserPosition,
    pub len: usize,
    pub data: AstNodeData,
}

// ---------------------------------------------------------------------------
// ExprASTError and ExprAST
// ---------------------------------------------------------------------------

/// ExprASTError — AST error information.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ExprASTError {
    pub msg: *const c_char,
    pub arg: *const c_char,
    pub arg_len: c_int,
}

/// ExprAST — the full AST for one parsed expression.
#[repr(C)]
pub struct ExprAST {
    pub err: ExprASTError,
    pub root: *mut ExprASTNode,
}
