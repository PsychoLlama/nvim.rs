//! The Vimscript expression parser: the engine behind `nvim_parse_expression`
//! and the cmdline highlighter.
//!
//! This file holds only the shared vocabulary — the token and AST node types,
//! and the integer enums the unit specs and `api/vimscript/` name. The work
//! is split across four submodules:
//!
//! - `lexer` scans one token at a time out of the parser's input.
//! - `ast` owns the node tables, the node arena and its slots, and the
//!   shunting-yard step that attaches a binary operator to the tree.
//! - `strings` decodes single- and double-quoted string literals.
//! - `parse` is the state machine that drives the other three, handing each
//!   token to a handler in `operators`, `values`, `brackets` or `figure`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

mod ast;
mod brackets;
mod figure;
mod lexer;
mod operators;
mod parse;
mod strings;
mod values;

pub use ast::{ASSIGNMENT_NAMES, CASE_STRATEGY_NAMES, COMPARISON_NAMES, NODE_TYPE_NAMES};
pub use lexer::viml_pexpr_next_token;
pub use parse::viml_pexpr_parse;

use crate::keycodes::{FSK_KEYCODE, FSK_SIMPLIFY};
use crate::os::libc::{size_t, uint8_t};
use ast::{Slot, viml_pexpr_handle_bop};
use strings::{parse_quoted_string, shifted_pos};

use crate::charset::hex2nr;
use crate::mbyte::utf_char2len;
use crate::types::{Float, UVarNumber};
use crate::viml::parser::parser::{ParserPosition, ParserState};
pub const CC_STRATEGY_IGNORE_CASE: ExprCaseCompareStrategy = 63;
pub const CC_STRATEGY_MATCH_CASE: ExprCaseCompareStrategy = 35;
pub const CC_STRATEGY_USE_OPTION: ExprCaseCompareStrategy = 0;
pub type LexExprTokenType = ::core::ffi::c_uint;
pub const EXPR_LEX_ASSIGNMENT: LexExprTokenType = 26;
pub const EXPR_LEX_ARROW: LexExprTokenType = 25;
pub const EXPR_LEX_COMMA: LexExprTokenType = 24;
pub const EXPR_LEX_PARENTHESIS: LexExprTokenType = 23;
pub const EXPR_LEX_FIGURE_BRACE: LexExprTokenType = 22;
pub const EXPR_LEX_BRACKET: LexExprTokenType = 21;
pub const EXPR_LEX_PLAIN_IDENTIFIER: LexExprTokenType = 20;
pub const EXPR_LEX_ENV: LexExprTokenType = 19;
pub const EXPR_LEX_REGISTER: LexExprTokenType = 18;
pub const EXPR_LEX_OPTION: LexExprTokenType = 17;
pub const EXPR_LEX_DOUBLE_QUOTED_STRING: LexExprTokenType = 16;
pub const EXPR_LEX_SINGLE_QUOTED_STRING: LexExprTokenType = 15;
pub const EXPR_LEX_NUMBER: LexExprTokenType = 14;
pub const EXPR_LEX_NOT: LexExprTokenType = 13;
pub const EXPR_LEX_MULTIPLICATION: LexExprTokenType = 12;
pub const EXPR_LEX_DOT: LexExprTokenType = 11;
pub const EXPR_LEX_MINUS: LexExprTokenType = 10;
pub const EXPR_LEX_PLUS: LexExprTokenType = 9;
pub const EXPR_LEX_COMPARISON: LexExprTokenType = 8;
pub const EXPR_LEX_AND: LexExprTokenType = 7;
pub const EXPR_LEX_OR: LexExprTokenType = 6;
pub const EXPR_LEX_COLON: LexExprTokenType = 5;
pub const EXPR_LEX_QUESTION: LexExprTokenType = 4;
pub const EXPR_LEX_EOC: LexExprTokenType = 3;
pub const EXPR_LEX_SPACING: LexExprTokenType = 2;
pub const EXPR_LEX_MISSING: LexExprTokenType = 1;
pub const EXPR_LEX_INVALID: LexExprTokenType = 0;
pub const EXPR_CMP_IDENTICAL: ExprComparisonType = 4;
pub const EXPR_CMP_GREATER_OR_EQUAL: ExprComparisonType = 3;
pub const EXPR_CMP_GREATER: ExprComparisonType = 2;
pub const EXPR_CMP_MATCHES: ExprComparisonType = 1;
pub const EXPR_CMP_EQUAL: ExprComparisonType = 0;
pub const EXPR_OPT_SCOPE_LOCAL: ExprOptScope = 108;
pub const EXPR_OPT_SCOPE_GLOBAL: ExprOptScope = 103;
pub const EXPR_OPT_SCOPE_UNSPECIFIED: ExprOptScope = 0;
pub const EXPR_ASGN_CONCAT: ExprAssignmentType = 3;
pub const EXPR_ASGN_SUBTRACT: ExprAssignmentType = 2;
pub const EXPR_ASGN_ADD: ExprAssignmentType = 1;
pub const EXPR_ASGN_PLAIN: ExprAssignmentType = 0;
pub const EXPR_VAR_SCOPE_ARGUMENTS: ExprVarScope = 97;
pub const EXPR_VAR_SCOPE_LOCAL: ExprVarScope = 108;
pub const EXPR_VAR_SCOPE_TABPAGE: ExprVarScope = 116;
pub const EXPR_VAR_SCOPE_WINDOW: ExprVarScope = 119;
pub const EXPR_VAR_SCOPE_BUFFER: ExprVarScope = 98;
pub const EXPR_VAR_SCOPE_VIM: ExprVarScope = 118;
pub const EXPR_VAR_SCOPE_GLOBAL: ExprVarScope = 103;
pub const EXPR_VAR_SCOPE_SCRIPT: ExprVarScope = 115;
pub const EXPR_VAR_SCOPE_MISSING: ExprVarScope = 0;
#[derive(Copy, Clone, Debug)]
pub struct LexExprToken {
    pub start: ParserPosition,
    pub len: size_t,
    pub type_0: LexExprTokenType,
    pub data: LexExprTokenData,
}
/// A lexed token's payload: the arm `LexExprToken::type_0` names, or
/// [`Blank`](LexExprTokenData::Blank) for the types that carry nothing.
///
/// The C is a union, and the parser reads the *wrong* member of it twice --
/// `values::option` asks an invalid option token for `opt.scope`, and
/// `operators::comparison` asks an invalid comparison for `cmp.ccs`, over
/// bytes the lexer wrote as `err`. Both answers reach the highlight list
/// `nvim_parse_expression` hands back, so both are behaviour. Neither is
/// garbage, though: the whole union starts zeroed (`blank_token`), `err`
/// covers only its first two members, and the one write that survives it is
/// the option scope -- which is why [`LexExprTokenError`] names that scope
/// outright. Every other arm answers its zero, which is what the C reads
/// there. See the accessors in `parse.rs`.
#[derive(Copy, Clone, Debug)]
pub enum LexExprTokenData {
    /// No payload: a token whose type says everything about it, and what
    /// every token starts as.
    Blank,
    Comparison(LexExprTokenComparison),
    Multiplication(LexExprTokenMultiplication),
    Brace(LexExprTokenBrace),
    Register(LexExprTokenRegister),
    Str(LexExprTokenString),
    Option(LexExprTokenOption),
    Var(LexExprTokenVar),
    Error(LexExprTokenError),
    Number(LexExprTokenNumber),
    Assignment(LexExprTokenAssignment),
}

#[derive(Copy, Clone, Debug)]
pub struct LexExprTokenAssignment {
    pub type_0: ExprAssignmentType,
}
/// A number literal: its value, and the base its prefix named -- which is
/// what decides how much of the token the `NumberPrefix` highlight covers.
#[derive(Copy, Clone, Debug)]
pub struct LexExprTokenNumber {
    pub val: LexExprTokenNumberValue,
    pub base: uint8_t,
}
/// A number literal's value.  The C pairs a union with the `is_float` flag
/// that selects it; the flag *is* the tag here.
#[derive(Copy, Clone, Debug)]
pub enum LexExprTokenNumberValue {
    Integer(UVarNumber),
    Floating(Float),
}
/// What an invalid token was trying to be, and why it is not.
#[derive(Copy, Clone, Debug)]
pub struct LexExprTokenError {
    pub type_0: LexExprTokenType,
    /// The translated message, with a `%.*s` where the rest of the line goes.
    pub msg: &'static ::core::ffi::CStr,
    /// The scope an invalid *option* token had already read out of `&g:`
    /// before its name turned out to be missing.
    ///
    /// The C keeps this by accident -- `err` overwrites the first two
    /// members of the union and `opt.scope` is the third, so the scope
    /// stands -- and `values::option` reads it back to decide whether to
    /// highlight a scope prefix. Carried rather than left to the layout.
    pub opt_scope: ExprOptScope,
}
#[derive(Copy, Clone, Debug)]
pub struct LexExprTokenVar {
    pub scope: ExprVarScope,
    pub autoload: bool,
}
#[derive(Copy, Clone, Debug)]
pub struct LexExprTokenOption {
    /// Where the name starts, counted from the start of the token: past the
    /// `&`, and past its `g:`/`l:` when there is one.
    pub name_offset: size_t,
    pub len: size_t,
    pub scope: ExprOptScope,
}
#[derive(Copy, Clone, Debug)]
pub struct LexExprTokenString {
    pub closed: bool,
}
#[derive(Copy, Clone, Debug)]
pub struct LexExprTokenRegister {
    pub name: ::core::ffi::c_int,
}
#[derive(Copy, Clone, Debug)]
pub struct LexExprTokenBrace {
    pub closing: bool,
}
#[derive(Copy, Clone, Debug)]
pub struct LexExprTokenMultiplication {
    pub type_0: ExprLexMulType,
}
pub type ExprLexMulType = ::core::ffi::c_uint;
pub const EXPR_LEX_MUL_MOD: ExprLexMulType = 2;
pub const EXPR_LEX_MUL_DIV: ExprLexMulType = 1;
pub const EXPR_LEX_MUL_MUL: ExprLexMulType = 0;
#[derive(Copy, Clone, Debug)]
pub struct LexExprTokenComparison {
    pub type_0: ExprComparisonType,
    pub ccs: ExprCaseCompareStrategy,
    pub inv: bool,
}
pub const EL_FLAG_FORBID_EOC: ::core::ffi::c_uint = 16;
pub const EL_FLAG_IS_NOT_CMP: ::core::ffi::c_uint = 8;
pub const EL_FLAG_ALLOW_FLOAT: ::core::ffi::c_uint = 4;
pub const EL_FLAG_FORBID_SCOPE: ::core::ffi::c_uint = 2;
pub const EL_FLAG_PEEK: ::core::ffi::c_uint = 1;
pub const EXPR_NODE_ASSIGNMENT: ExprASTNodeType = 38;
pub const EXPR_NODE_ENVIRONMENT: ExprASTNodeType = 37;
pub const EXPR_NODE_OPTION: ExprASTNodeType = 36;
pub const EXPR_NODE_MOD: ExprASTNodeType = 35;
pub const EXPR_NODE_DIVISION: ExprASTNodeType = 34;
pub const EXPR_NODE_MULTIPLICATION: ExprASTNodeType = 33;
pub const EXPR_NODE_NOT: ExprASTNodeType = 32;
pub const EXPR_NODE_BINARY_MINUS: ExprASTNodeType = 31;
pub const EXPR_NODE_UNARY_MINUS: ExprASTNodeType = 30;
pub const EXPR_NODE_AND: ExprASTNodeType = 29;
pub const EXPR_NODE_OR: ExprASTNodeType = 28;
pub const EXPR_NODE_DOUBLE_QUOTED_STRING: ExprASTNodeType = 27;
pub const EXPR_NODE_SINGLE_QUOTED_STRING: ExprASTNodeType = 26;
pub const EXPR_NODE_FLOAT: ExprASTNodeType = 25;
pub const EXPR_NODE_INTEGER: ExprASTNodeType = 24;
pub const EXPR_NODE_CONCAT_OR_SUBSCRIPT: ExprASTNodeType = 23;
pub const EXPR_NODE_CONCAT: ExprASTNodeType = 22;
pub const EXPR_NODE_COMPARISON: ExprASTNodeType = 21;
pub const EXPR_NODE_ARROW: ExprASTNodeType = 20;
pub const EXPR_NODE_COLON: ExprASTNodeType = 19;
pub const EXPR_NODE_COMMA: ExprASTNodeType = 18;
pub const EXPR_NODE_CURLY_BRACES_IDENTIFIER: ExprASTNodeType = 17;
pub const EXPR_NODE_DICT_LITERAL: ExprASTNodeType = 16;
pub const EXPR_NODE_LAMBDA: ExprASTNodeType = 15;
pub const EXPR_NODE_UNKNOWN_FIGURE: ExprASTNodeType = 14;
pub const EXPR_NODE_COMPLEX_IDENTIFIER: ExprASTNodeType = 13;
pub const EXPR_NODE_PLAIN_KEY: ExprASTNodeType = 12;
pub const EXPR_NODE_PLAIN_IDENTIFIER: ExprASTNodeType = 11;
pub const EXPR_NODE_CALL: ExprASTNodeType = 10;
pub const EXPR_NODE_NESTED: ExprASTNodeType = 9;
pub const EXPR_NODE_BINARY_PLUS: ExprASTNodeType = 8;
pub const EXPR_NODE_UNARY_PLUS: ExprASTNodeType = 7;
pub const EXPR_NODE_LIST_LITERAL: ExprASTNodeType = 6;
pub const EXPR_NODE_SUBSCRIPT: ExprASTNodeType = 5;
pub const EXPR_NODE_REGISTER: ExprASTNodeType = 4;
pub const EXPR_NODE_TERNARY_VALUE: ExprASTNodeType = 3;
pub const EXPR_NODE_TERNARY: ExprASTNodeType = 2;
pub const EXPR_NODE_OP_MISSING: ExprASTNodeType = 1;
pub const EXPR_NODE_MISSING: ExprASTNodeType = 0;
pub const EXPR_FLAGS_PARSE_LET: ExprParserFlags = 4;
pub const EXPR_FLAGS_DISALLOW_EOC: ExprParserFlags = 2;
pub const EXPR_FLAGS_MULTI: ExprParserFlags = 1;
pub const EPT_LAMBDA_ARGUMENTS: ExprASTParseType = 1;
pub type ExprASTParseType = ::core::ffi::c_uint;
pub const EPT_SINGLE_ASSIGNMENT: ExprASTParseType = 3;
pub const EPT_ASSIGNMENT: ExprASTParseType = 2;
pub const EPT_EXPR: ExprASTParseType = 0;
pub const E_NODE_VALUE: ExprASTWantedNode = 1;
pub type ExprASTWantedNode = ::core::ffi::c_uint;
pub const E_NODE_OPERATOR: ExprASTWantedNode = 0;
pub const E_OP_ASS_RIGHT: ExprOpAssociativity = 114;
pub type ExprOpAssociativity = ::core::ffi::c_uint;
pub const E_OP_ASS_LEFT: ExprOpAssociativity = 108;
pub const E_OP_ASS_NO: ExprOpAssociativity = 110;
pub type ExprOpLvl = ::core::ffi::c_uint;
pub const E_OP_LVL_VALUE: ExprOpLvl = 16;
pub const E_OP_LVL_SUBSCRIPT: ExprOpLvl = 15;
pub const E_OP_LVL_UNARY: ExprOpLvl = 14;
pub const E_OP_LVL_MULTIPLICATION: ExprOpLvl = 13;
pub const E_OP_LVL_ADDITION: ExprOpLvl = 12;
pub const E_OP_LVL_COMPARISON: ExprOpLvl = 11;
pub const E_OP_LVL_AND: ExprOpLvl = 10;
pub const E_OP_LVL_OR: ExprOpLvl = 9;
pub const E_OP_LVL_TERNARY: ExprOpLvl = 8;
pub const E_OP_LVL_TERNARY_VALUE: ExprOpLvl = 7;
pub const E_OP_LVL_COLON: ExprOpLvl = 6;
pub const E_OP_LVL_COMMA: ExprOpLvl = 5;
pub const E_OP_LVL_ARROW: ExprOpLvl = 4;
pub const E_OP_LVL_ASSIGNMENT: ExprOpLvl = 3;
pub const E_OP_LVL_PARENS: ExprOpLvl = 2;
pub const E_OP_LVL_COMPLEX_IDENTIFIER: ExprOpLvl = 1;
pub const E_OP_LVL_INVALID: ExprOpLvl = 0;
#[derive(Debug)]
pub struct ExprNodeProps {
    pub lvl: ExprOpLvl,
    pub ass: ExprOpAssociativity,
}
#[derive(Debug)]
pub struct StringShift {
    pub start: size_t,
    pub orig_len: size_t,
    pub act_len: size_t,
    pub escape_not_known: bool,
}
/// The slot `back` places down from the top of the AST stack; `back` of zero
/// is the top. Panics on an empty stack, where the C read one slot before the
/// buffer.
fn stack_top(stack: &[Slot], back: usize) -> Slot {
    stack[stack.len() - 1 - back]
}

/// A parsed `:echo`-style expression: the error, if the parse found one, and
/// the tree.
///
/// The nodes live in one arena and name each other by [`NodeId`]: the parser
/// splices operators in above values it has already built, which an owned
/// tree of boxes could only do by walking down from the root each time. The
/// tree borrows the parsed lines, which its identifiers and its error point
/// into.
#[derive(Debug)]
pub struct ExprAST<'a> {
    pub err: Option<ExprASTError<'a>>,
    pub nodes: Vec<ExprASTNode<'a>>,
    pub root: Option<NodeId>,
}

/// A node of an [`ExprAST`], by its place in the arena.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NodeId(pub(crate) usize);

/// Where a parse stopped, and why.
#[derive(Copy, Clone, Debug)]
pub struct ExprASTError<'a> {
    /// The translated message, with a `%.*s` where `arg` goes.
    pub msg: &'static ::core::ffi::CStr,
    /// The rest of the line from where the error was found; `None` when the
    /// error is past the last line.
    pub arg: Option<&'a [u8]>,
}
pub type ExprASTNodeType = ::core::ffi::c_uint;
pub type ExprAssignmentType = ::core::ffi::c_uint;
pub type ExprCaseCompareStrategy = ::core::ffi::c_uint;
pub type ExprComparisonType = ::core::ffi::c_uint;
pub type ExprOptScope = ::core::ffi::c_uint;
pub type ExprParserFlags = ::core::ffi::c_uint;
pub type ExprVarScope = ::core::ffi::c_uint;
#[derive(Debug)]
pub struct ExprASTNode<'a> {
    pub type_0: ExprASTNodeType,
    /// The first child; the second, if any, is its `next`.
    pub children: Option<NodeId>,
    /// The next sibling.
    pub next: Option<NodeId>,
    pub start: ParserPosition,
    pub len: size_t,
    pub data: ExprNodeData<'a>,
}
/// The payload of an [`ExprASTNode`], as its [`ExprASTNodeType`] selects.
///
/// Upstream this is a union with no tag of its own: the node's `type_0` says
/// which member is live and every read has to agree. Here the tag is the
/// enum's, so reading the wrong arm is a panic rather than a
/// reinterpretation, and `type_0` and the payload can no longer disagree
/// silently. The node types that carry nothing get [`ExprNodeData::None`],
/// which is also what a freshly allocated node starts as -- upstream left
/// those bytes uninitialised.
#[derive(Clone, Debug)]
pub enum ExprNodeData<'a> {
    /// Every node type that carries no payload.
    None,
    /// `EXPR_NODE_REGISTER`.
    Register(ExprNodeRegister),
    /// `EXPR_NODE_UNKNOWN_FIGURE`, `EXPR_NODE_LAMBDA`, `EXPR_NODE_DICT_LITERAL`
    /// and `EXPR_NODE_CURLY_BRACES_IDENTIFIER`.
    Figure(ExprNodeFigure),
    /// `EXPR_NODE_PLAIN_IDENTIFIER` and `EXPR_NODE_PLAIN_KEY`.
    Variable(ExprNodeVariable<'a>),
    /// `EXPR_NODE_TERNARY_VALUE`.
    Ternary(ExprNodeTernary),
    /// `EXPR_NODE_COMPARISON`.
    Comparison(ExprNodeComparison),
    /// `EXPR_NODE_INTEGER`.
    Integer(ExprNodeInteger),
    /// `EXPR_NODE_FLOAT`.
    Float(ExprNodeFloat),
    /// `EXPR_NODE_SINGLE_QUOTED_STRING` and `EXPR_NODE_DOUBLE_QUOTED_STRING`.
    Str(ExprNodeStr),
    /// `EXPR_NODE_OPTION`.
    Opt(ExprNodeOption<'a>),
    /// `EXPR_NODE_ENVIRONMENT`.
    Environment(ExprNodeEnvironment<'a>),
    /// `EXPR_NODE_ASSIGNMENT`.
    Assignment(ExprNodeAssignment),
}

impl<'a> ExprNodeData<'a> {
    /// The payload of a figure-brace node.
    #[track_caller]
    pub fn figure(&self) -> &ExprNodeFigure {
        match self {
            Self::Figure(v) => v,
            _ => panic!("node payload is not a figure brace"),
        }
    }

    /// The payload of a ternary-value node.
    #[track_caller]
    pub fn ternary(&self) -> &ExprNodeTernary {
        match self {
            Self::Ternary(v) => v,
            _ => panic!("node payload is not a ternary value"),
        }
    }

    /// The payload of a string-literal node.
    #[track_caller]
    pub fn string(&self) -> &ExprNodeStr {
        match self {
            Self::Str(v) => v,
            _ => panic!("node payload is not a string literal"),
        }
    }

    /// The payload of an option node.
    #[track_caller]
    pub fn option(&self) -> &ExprNodeOption<'a> {
        match self {
            Self::Opt(v) => v,
            _ => panic!("node payload is not an option"),
        }
    }

    /// The payload of an identifier or key node.
    #[track_caller]
    pub fn variable(&self) -> &ExprNodeVariable<'a> {
        match self {
            Self::Variable(v) => v,
            _ => panic!("node payload is not an identifier"),
        }
    }

    /// The payload of an environment-variable node.
    #[track_caller]
    pub fn environment(&self) -> &ExprNodeEnvironment<'a> {
        match self {
            Self::Environment(v) => v,
            _ => panic!("node payload is not an environment variable"),
        }
    }

    /// The payload of a register node.
    #[track_caller]
    pub fn register(&self) -> &ExprNodeRegister {
        match self {
            Self::Register(v) => v,
            _ => panic!("node payload is not a register"),
        }
    }

    /// The payload of a comparison node.
    #[track_caller]
    pub fn comparison(&self) -> &ExprNodeComparison {
        match self {
            Self::Comparison(v) => v,
            _ => panic!("node payload is not a comparison"),
        }
    }

    /// The payload of an integer-literal node.
    #[track_caller]
    pub fn integer(&self) -> &ExprNodeInteger {
        match self {
            Self::Integer(v) => v,
            _ => panic!("node payload is not an integer literal"),
        }
    }

    /// The payload of a float-literal node.
    #[track_caller]
    pub fn float(&self) -> &ExprNodeFloat {
        match self {
            Self::Float(v) => v,
            _ => panic!("node payload is not a float literal"),
        }
    }

    /// The payload of an assignment node.
    #[track_caller]
    pub fn assignment(&self) -> &ExprNodeAssignment {
        match self {
            Self::Assignment(v) => v,
            _ => panic!("node payload is not an assignment"),
        }
    }
}
#[derive(Copy, Clone, Debug)]
pub struct ExprNodeAssignment {
    pub type_0: ExprAssignmentType,
}
#[derive(Copy, Clone, Debug)]
pub struct ExprNodeComparison {
    pub type_0: ExprComparisonType,
    pub ccs: ExprCaseCompareStrategy,
    pub inv: bool,
}
#[derive(Copy, Clone, Debug)]
pub struct ExprNodeEnvironment<'a> {
    pub ident: &'a [u8],
}
#[derive(Copy, Clone, Debug)]
pub struct ExprNodeFigure {
    pub type_guesses: ExprFigureGuesses,
    pub opening_hl_idx: size_t,
}
#[derive(Copy, Clone, Debug)]
pub struct ExprFigureGuesses {
    pub allow_dict: bool,
    pub allow_lambda: bool,
    pub allow_ident: bool,
}
#[derive(Copy, Clone, Debug)]
pub struct ExprNodeFloat {
    pub value: Float,
}
#[derive(Copy, Clone, Debug)]
pub struct ExprNodeInteger {
    pub value: UVarNumber,
}
#[derive(Copy, Clone, Debug)]
pub struct ExprNodeOption<'a> {
    pub ident: &'a [u8],
    pub scope: ExprOptScope,
}
#[derive(Copy, Clone, Debug)]
pub struct ExprNodeRegister {
    pub name: ::core::ffi::c_int,
}
/// A string literal's decoded value. `None` where upstream allocated no
/// buffer at all -- an empty single-quoted literal, or a double-quoted one
/// whose reserve came to nothing -- which is not the same as an empty one.
#[derive(Clone, Debug)]
pub struct ExprNodeStr {
    pub value: Option<Vec<u8>>,
}
#[derive(Copy, Clone, Debug)]
pub struct ExprNodeTernary {
    pub got_colon: bool,
}
#[derive(Copy, Clone, Debug)]
pub struct ExprNodeVariable<'a> {
    pub scope: ExprVarScope,
    pub ident: &'a [u8],
}
