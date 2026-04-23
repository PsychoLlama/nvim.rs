/// Exported string-table arrays for the VimL expression parser.
///
/// These replace the C definitions that were previously in expressions.c.
/// They are exported with their original C names so that C callers
/// (vimscript.c, ex_getln.c, test code) continue to link without changes.
use std::ffi::c_char;

/// Helper: produce a `*const c_char` pointing to the NUL-terminated literal.
macro_rules! cstr {
    ($s:literal) => {
        concat!($s, "\0").as_ptr().cast::<c_char>()
    };
}

/// Wrapper that makes a raw `*const c_char` pointer Sync+Send.
///
/// The pointers always point to static string-literal data in the binary's
/// read-only segment, so sharing across threads is sound.
#[repr(transparent)]
struct CStrPtr(*const c_char);

// SAFETY: all pointers stored here refer to immutable string literals.
unsafe impl Sync for CStrPtr {}
unsafe impl Send for CStrPtr {}

// ---------------------------------------------------------------------------
// east_node_type_tab
// ---------------------------------------------------------------------------

/// Array mapping ExprASTNodeType values to their stringified versions.
///
/// Order must match the ExprASTNodeType enum in expressions.h / expr_types.rs.
/// Exported with the original C name so callers link without changes.
#[no_mangle]
static east_node_type_tab: [CStrPtr; 39] = [
    CStrPtr(cstr!("Missing")),               // kExprNodeMissing = 0
    CStrPtr(cstr!("OpMissing")),             // kExprNodeOpMissing
    CStrPtr(cstr!("Ternary")),               // kExprNodeTernary
    CStrPtr(cstr!("TernaryValue")),          // kExprNodeTernaryValue
    CStrPtr(cstr!("Register")),              // kExprNodeRegister
    CStrPtr(cstr!("Subscript")),             // kExprNodeSubscript
    CStrPtr(cstr!("ListLiteral")),           // kExprNodeListLiteral
    CStrPtr(cstr!("UnaryPlus")),             // kExprNodeUnaryPlus
    CStrPtr(cstr!("BinaryPlus")),            // kExprNodeBinaryPlus
    CStrPtr(cstr!("Nested")),                // kExprNodeNested
    CStrPtr(cstr!("Call")),                  // kExprNodeCall
    CStrPtr(cstr!("PlainIdentifier")),       // kExprNodePlainIdentifier
    CStrPtr(cstr!("PlainKey")),              // kExprNodePlainKey
    CStrPtr(cstr!("ComplexIdentifier")),     // kExprNodeComplexIdentifier
    CStrPtr(cstr!("UnknownFigure")),         // kExprNodeUnknownFigure
    CStrPtr(cstr!("Lambda")),                // kExprNodeLambda
    CStrPtr(cstr!("DictLiteral")),           // kExprNodeDictLiteral
    CStrPtr(cstr!("CurlyBracesIdentifier")), // kExprNodeCurlyBracesIdentifier
    CStrPtr(cstr!("Comma")),                 // kExprNodeComma
    CStrPtr(cstr!("Colon")),                 // kExprNodeColon
    CStrPtr(cstr!("Arrow")),                 // kExprNodeArrow
    CStrPtr(cstr!("Comparison")),            // kExprNodeComparison
    CStrPtr(cstr!("Concat")),                // kExprNodeConcat
    CStrPtr(cstr!("ConcatOrSubscript")),     // kExprNodeConcatOrSubscript
    CStrPtr(cstr!("Integer")),               // kExprNodeInteger
    CStrPtr(cstr!("Float")),                 // kExprNodeFloat
    CStrPtr(cstr!("SingleQuotedString")),    // kExprNodeSingleQuotedString
    CStrPtr(cstr!("DoubleQuotedString")),    // kExprNodeDoubleQuotedString
    CStrPtr(cstr!("Or")),                    // kExprNodeOr
    CStrPtr(cstr!("And")),                   // kExprNodeAnd
    CStrPtr(cstr!("UnaryMinus")),            // kExprNodeUnaryMinus
    CStrPtr(cstr!("BinaryMinus")),           // kExprNodeBinaryMinus
    CStrPtr(cstr!("Not")),                   // kExprNodeNot
    CStrPtr(cstr!("Multiplication")),        // kExprNodeMultiplication
    CStrPtr(cstr!("Division")),              // kExprNodeDivision
    CStrPtr(cstr!("Mod")),                   // kExprNodeMod
    CStrPtr(cstr!("Option")),                // kExprNodeOption
    CStrPtr(cstr!("Environment")),           // kExprNodeEnvironment
    CStrPtr(cstr!("Assignment")),            // kExprNodeAssignment
];

// ---------------------------------------------------------------------------
// eltkn_cmp_type_tab
// ---------------------------------------------------------------------------

/// Array mapping ExprComparisonType values to their stringified versions.
#[no_mangle]
static eltkn_cmp_type_tab: [CStrPtr; 5] = [
    CStrPtr(cstr!("Equal")),          // kExprCmpEqual = 0
    CStrPtr(cstr!("Matches")),        // kExprCmpMatches
    CStrPtr(cstr!("Greater")),        // kExprCmpGreater
    CStrPtr(cstr!("GreaterOrEqual")), // kExprCmpGreaterOrEqual
    CStrPtr(cstr!("Identical")),      // kExprCmpIdentical
];

// ---------------------------------------------------------------------------
// ccs_tab
// ---------------------------------------------------------------------------

/// Array mapping ExprCaseCompareStrategy values to their stringified versions.
///
/// The C enum uses character values as discriminants:
///   kCCStrategyUseOption = 0
///   kCCStrategyMatchCase = '#' = 35
///   kCCStrategyIgnoreCase = '?' = 63
///
/// C code indexes this array directly with the enum value, so the array must
/// have at least 64 entries and entries at positions 0, 35, and 63 filled.
#[no_mangle]
static ccs_tab: [CStrPtr; 64] = {
    // All-null array; we override three slots below.
    let mut arr: [CStrPtr; 64] = [const { CStrPtr(std::ptr::null()) }; 64];
    arr[0] = CStrPtr(cstr!("UseOption")); // kCCStrategyUseOption
    arr[35] = CStrPtr(cstr!("MatchCase")); // kCCStrategyMatchCase = '#'
    arr[63] = CStrPtr(cstr!("IgnoreCase")); // kCCStrategyIgnoreCase = '?'
    arr
};

// ---------------------------------------------------------------------------
// expr_asgn_type_tab
// ---------------------------------------------------------------------------

/// Array mapping ExprAssignmentType values to their stringified versions.
#[no_mangle]
static expr_asgn_type_tab: [CStrPtr; 4] = [
    CStrPtr(cstr!("Plain")),    // kExprAsgnPlain = 0
    CStrPtr(cstr!("Add")),      // kExprAsgnAdd
    CStrPtr(cstr!("Subtract")), // kExprAsgnSubtract
    CStrPtr(cstr!("Concat")),   // kExprAsgnConcat
];

// ---------------------------------------------------------------------------
// node_maxchildren
// ---------------------------------------------------------------------------

/// Array mapping ExprASTNodeType to maximum number of children a node may have.
///
/// Accessible from other Rust modules as `NODE_MAXCHILDREN`.
pub const NODE_MAXCHILDREN: [u8; 39] = [
    0, // kExprNodeMissing
    2, // kExprNodeOpMissing
    2, // kExprNodeTernary
    2, // kExprNodeTernaryValue
    0, // kExprNodeRegister
    2, // kExprNodeSubscript
    1, // kExprNodeListLiteral
    1, // kExprNodeUnaryPlus
    2, // kExprNodeBinaryPlus
    1, // kExprNodeNested
    2, // kExprNodeCall
    0, // kExprNodePlainIdentifier
    0, // kExprNodePlainKey
    2, // kExprNodeComplexIdentifier
    1, // kExprNodeUnknownFigure
    2, // kExprNodeLambda
    1, // kExprNodeDictLiteral
    1, // kExprNodeCurlyBracesIdentifier
    2, // kExprNodeComma
    2, // kExprNodeColon
    2, // kExprNodeArrow
    2, // kExprNodeComparison
    2, // kExprNodeConcat
    2, // kExprNodeConcatOrSubscript
    0, // kExprNodeInteger
    0, // kExprNodeFloat
    0, // kExprNodeSingleQuotedString
    0, // kExprNodeDoubleQuotedString
    2, // kExprNodeOr
    2, // kExprNodeAnd
    1, // kExprNodeUnaryMinus
    2, // kExprNodeBinaryMinus
    1, // kExprNodeNot
    2, // kExprNodeMultiplication
    2, // kExprNodeDivision
    2, // kExprNodeMod
    0, // kExprNodeOption
    0, // kExprNodeEnvironment
    2, // kExprNodeAssignment
];

#[no_mangle]
static node_maxchildren: [u8; 39] = [
    0, // kExprNodeMissing
    2, // kExprNodeOpMissing
    2, // kExprNodeTernary
    2, // kExprNodeTernaryValue
    0, // kExprNodeRegister
    2, // kExprNodeSubscript
    1, // kExprNodeListLiteral
    1, // kExprNodeUnaryPlus
    2, // kExprNodeBinaryPlus
    1, // kExprNodeNested
    2, // kExprNodeCall
    0, // kExprNodePlainIdentifier
    0, // kExprNodePlainKey
    2, // kExprNodeComplexIdentifier
    1, // kExprNodeUnknownFigure
    2, // kExprNodeLambda
    1, // kExprNodeDictLiteral
    1, // kExprNodeCurlyBracesIdentifier
    2, // kExprNodeComma
    2, // kExprNodeColon
    2, // kExprNodeArrow
    2, // kExprNodeComparison
    2, // kExprNodeConcat
    2, // kExprNodeConcatOrSubscript
    0, // kExprNodeInteger
    0, // kExprNodeFloat
    0, // kExprNodeSingleQuotedString
    0, // kExprNodeDoubleQuotedString
    2, // kExprNodeOr
    2, // kExprNodeAnd
    1, // kExprNodeUnaryMinus
    2, // kExprNodeBinaryMinus
    1, // kExprNodeNot
    2, // kExprNodeMultiplication
    2, // kExprNodeDivision
    2, // kExprNodeMod
    0, // kExprNodeOption
    0, // kExprNodeEnvironment
    2, // kExprNodeAssignment
];
