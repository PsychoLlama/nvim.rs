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

/// A parsed `:echo`-style expression: the error, if the parse found one, and
/// the tree.
///
/// The nodes live in one arena and name each other by [`NodeId`]: the parser
/// splices operators in above values it has already built, which an owned
/// tree of boxes could only do by walking down from the root each time. The
/// tree borrows the parsed lines, which its identifiers and its error point
/// into.
pub struct ExprAST<'a> {
    pub err: Option<ExprASTError<'a>>,
    pub nodes: Vec<ExprASTNode<'a>>,
    pub root: Option<NodeId>,
}

/// A node of an [`ExprAST`], by its place in the arena.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NodeId(pub(crate) usize);

/// Where a parse stopped, and why.
#[derive(Copy, Clone)]
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
#[derive(Clone)]
pub enum ExprNodeData<'a> {
    /// Every node type that carries no payload.
    None,
    /// `kExprNodeRegister`.
    Register(ExprNodeRegister),
    /// `kExprNodeUnknownFigure`, `kExprNodeLambda`, `kExprNodeDictLiteral`
    /// and `kExprNodeCurlyBracesIdentifier`.
    Figure(ExprNodeFigure),
    /// `kExprNodePlainIdentifier` and `kExprNodePlainKey`.
    Variable(ExprNodeVariable<'a>),
    /// `kExprNodeTernaryValue`.
    Ternary(ExprNodeTernary),
    /// `kExprNodeComparison`.
    Comparison(ExprNodeComparison),
    /// `kExprNodeInteger`.
    Integer(ExprNodeInteger),
    /// `kExprNodeFloat`.
    Float(ExprNodeFloat),
    /// `kExprNodeSingleQuotedString` and `kExprNodeDoubleQuotedString`.
    Str(ExprNodeStr),
    /// `kExprNodeOption`.
    Opt(ExprNodeOption<'a>),
    /// `kExprNodeEnvironment`.
    Environment(ExprNodeEnvironment<'a>),
    /// `kExprNodeAssignment`.
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
#[derive(Copy, Clone)]
pub struct ExprNodeAssignment {
    pub type_0: ExprAssignmentType,
}
#[derive(Copy, Clone)]
pub struct ExprNodeComparison {
    pub type_0: ExprComparisonType,
    pub ccs: ExprCaseCompareStrategy,
    pub inv: bool,
}
#[derive(Copy, Clone)]
pub struct ExprNodeEnvironment<'a> {
    pub ident: &'a [u8],
}
#[derive(Copy, Clone)]
pub struct ExprNodeFigure {
    pub type_guesses: ExprFigureGuesses,
    pub opening_hl_idx: size_t,
}
#[derive(Copy, Clone)]
pub struct ExprFigureGuesses {
    pub allow_dict: bool,
    pub allow_lambda: bool,
    pub allow_ident: bool,
}
#[derive(Copy, Clone)]
pub struct ExprNodeFloat {
    pub value: Float,
}
#[derive(Copy, Clone)]
pub struct ExprNodeInteger {
    pub value: UVarNumber,
}
#[derive(Copy, Clone)]
pub struct ExprNodeOption<'a> {
    pub ident: &'a [u8],
    pub scope: ExprOptScope,
}
#[derive(Copy, Clone)]
pub struct ExprNodeRegister {
    pub name: ::core::ffi::c_int,
}
/// A string literal's decoded value. `None` where upstream allocated no
/// buffer at all -- an empty single-quoted literal, or a double-quoted one
/// whose reserve came to nothing -- which is not the same as an empty one.
#[derive(Clone)]
pub struct ExprNodeStr {
    pub value: Option<Vec<u8>>,
}
#[derive(Copy, Clone)]
pub struct ExprNodeTernary {
    pub got_colon: bool,
}
#[derive(Copy, Clone)]
pub struct ExprNodeVariable<'a> {
    pub scope: ExprVarScope,
    pub ident: &'a [u8],
}
